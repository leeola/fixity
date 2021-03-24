use {
    crate::{
        cache::{ArchivedStructured, CacheRead, CacheWrite, OwnedRef, Structured},
        storage::{Error, StorageRead, StorageWrite},
        Addr,
    },
    log::warn,
    rkyv::{
        archived_value,
        de::deserializers::AllocDeserializer,
        ser::{serializers::WriteSerializer, Serializer},
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
}
#[async_trait::async_trait]
impl<S> CacheRead for ArchiveCache<S>
where
    S: StorageRead + Send,
{
    type OwnedRef = ArchiveBytes;
    async fn read_unstructured<A, W>(&self, addr: A, mut w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let addr_ref = addr.as_ref();
        {
            let buf = {
                let cache = self.cache.lock().map_err(|_| Error::Unhandled {
                    message: "cache mutex poisoned".to_owned(),
                })?;
                cache.get(addr_ref).map(Arc::clone)
            };
            if let Some(buf) = buf {
                return Ok(io::copy(&mut buf.as_slice(), &mut w).await?);
            }
        }
        // we could have a concurrency issue here, where we read from storage twice.
        // This is no-risk (ie won't corrupt data/etc), and should be tweaked based on
        // what results in better performance.
        // Optimizing for duplicate cache inserts vs holding the lock longer.
        // Possibly even keeping some type of LockState to have short lock length?
        // /shrug, bench concern for down the road.
        let mut buf = Vec::new();
        StorageRead::read(&self.storage, addr_ref.clone(), &mut buf).await?;
        let len = io::copy(&mut buf.as_slice(), &mut w).await?;
        let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
            message: "cache mutex poisoned".to_owned(),
        })?;
        if let Some(_) = cache.insert(addr_ref.clone(), Arc::new(buf)) {
            warn!("cache inserted twice, wasted storage read");
        }
        Ok(len)
    }
    async fn read_structured<A>(&self, addr: A) -> Result<Self::OwnedRef, Error>
    where
        A: AsRef<Addr> + Send,
    {
        todo!("read_struct")
    }
}
pub struct ArchiveBytes(Arc<Vec<u8>>);
impl OwnedRef for ArchiveBytes {
    type Ref = ArchivedStructured;
    fn as_ref(&self) -> &Self::Ref {
        let mut serializer = WriteSerializer::new(Vec::new());
        let buf = serializer.into_inner();
        // we're only serializing to the beginning of the buf, currently.
        let archived = unsafe { archived_value::<Structured>(buf.as_ref(), 0) };
        todo!("ArchiveBytes::as_ref")
    }
    fn into_owned(self) -> Structured {
        todo!("ArchiveBytes::into_owned")
    }
}
#[async_trait::async_trait]
impl<S> CacheWrite for ArchiveCache<S>
where
    S: StorageWrite,
{
    async fn write_unstructured<R>(&self, r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        todo!("write")
    }
    async fn write_structured<T>(&self, structured: T) -> Result<Addr, Error>
    where
        T: Into<Structured> + Send,
    {
        todo!("write_struct")
    }
}
