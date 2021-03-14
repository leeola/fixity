use {
    crate::{
        cache::{ArchivedStructured, CacheRead, CacheWrite, OwnedRef, Structured},
        deser::Deser,
        storage::{Error, StorageRead, StorageWrite},
        Addr,
    },
    log::warn,
    rkyv::{
        archived_value,
        de::deserializers::AllocDeserializer,
        ser::{serializers::WriteSerializer, SeekSerializer, Serializer},
        std_impl::{ArchivedString, ArchivedVec},
        Archive, Deserialize, Serialize,
    },
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    tokio::io::{self, AsyncRead, AsyncWrite},
};
pub struct ArchiveCache<S> {
    storage: S,
    // TODO: use an LRU or something useful. This is just a simple test of Caching + Archive.
    // TODO: use a RwLock here. Or ideally a lock-free data structure.
    cache: Mutex<HashMap<Addr, Arc<Vec<u8>>>>,
}
impl<S> ArchiveCache<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            cache: Mutex::new(HashMap::new()),
        }
    }
    /// Write the provided buf to the cache, and internal storage if needed.
    async fn write_buf(&self, addr: &Addr, buf: Vec<u8>) -> Result<(), Error>
    where
        S: StorageWrite + Send,
    {
        let buf = Arc::new(buf);
        let new_to_cache = {
            let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            cache.insert(addr.clone(), Arc::clone(&buf)).is_none()
        };
        // as an optimization, if it's already in the memory cache we should be able to ignore
        // writing it to storage.
        if new_to_cache {
            let _: u64 = self
                .storage
                .write(addr.clone(), &mut buf.as_slice())
                .await?;
        }
        Ok(())
    }
    async fn read_buf<A>(&self, addr: A) -> Result<Arc<Vec<u8>>, Error>
    where
        S: StorageRead + Send,
        A: AsRef<Addr> + Into<Addr> + Send,
    {
        let addr_ref = addr.as_ref();
        {
            let cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            let buf = cache.get(addr_ref).map(Arc::clone);
            if let Some(buf) = buf {
                return Ok(Arc::clone(&buf));
            }
        }
        // we could have a concurrency issue here, where we read from storage twice.
        // This is low-risk (ie won't corrupt data/etc), and should be tweaked based on
        // what results in better performance.
        // Optimizing for duplicate cache inserts vs holding the lock longer.
        // Possibly even keeping some type of LockState to have short lock length?
        // /shrug, bench concern for down the road.
        let mut buf = Vec::new();
        let _: u64 = StorageRead::read(&self.storage, addr_ref.clone(), &mut buf).await?;
        let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
            message: "cache mutex poisoned".to_owned(),
        })?;
        let buf = Arc::new(buf);
        if let Some(_) = cache.insert(addr_ref.clone(), Arc::clone(&buf)) {
            warn!("cache inserted twice, needless storage read");
        }
        Ok(buf)
    }
}
#[async_trait::async_trait]
impl<S> CacheRead for ArchiveCache<S>
where
    S: StorageRead + Send,
{
    type OwnedRef = ArchiveBytes;
    async fn read_unstructured<A, W>(&self, addr: A, mut w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let buf = self.read_buf(addr).await?;
        let len = io::copy(&mut buf.as_slice(), &mut w).await?;
        Ok(len)
    }
    async fn read_structured<A>(&self, addr: A) -> Result<Self::OwnedRef, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send,
    {
        let buf = self.read_buf(addr).await?;
        Ok(ArchiveBytes(buf))
    }
}
pub struct ArchiveBytes(Arc<Vec<u8>>);
impl OwnedRef for ArchiveBytes {
    fn as_ref(&self) -> &ArchivedStructured {
        unsafe {
            archived_value::<Structured>(
                self.0.as_slice(),
                // we're only serializing to the beginning of the buf.
                0,
            )
        }
    }
    fn into_owned(self) -> Structured {
        let archived = self.as_ref();
        let mut deserializer = AllocDeserializer;
        let deserialized = archived.deserialize(&mut deserializer).unwrap();
        deserialized
    }
}
#[async_trait::async_trait]
impl<S> CacheWrite for ArchiveCache<S>
where
    S: StorageWrite + Send,
{
    async fn write_unstructured<R>(&self, mut r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let _: u64 = io::copy(&mut r, &mut buf).await?;
        let addr = Addr::hash(&buf);
        self.write_buf(&addr, buf).await?;
        Ok(addr)
    }
    async fn write_structured<T>(&self, structured: T) -> Result<Addr, Error>
    where
        T: Into<Structured> + Send,
    {
        let structured = structured.into();
        let addr = {
            let deser_buf = Deser::default().to_vec(&structured).unwrap();
            Addr::hash(&deser_buf)
        };
        let mut serializer = WriteSerializer::new(std::io::Cursor::new(Vec::new()));
        let pos: usize = serializer
            .archive_root(&structured)
            .map_err(|err| Error::Unhandled {
                message: format!("Archive serialization: {}", err),
            })?;
        if pos != 0 {
            return Err(Error::Unhandled {
                message: "archive position unexpectedly not zero".to_owned(),
            });
        }
        let buf = serializer.into_inner().into_inner();
        self.write_buf(&addr, buf).await?;
        Ok(addr)
    }
}
