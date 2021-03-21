use {
    crate::{
        cache::{ArchivedStructured, CacheRead, CacheWrite, Structured},
        storage::{Error, StorageRead, StorageWrite},
        Addr,
    },
    log::warn,
    std::{collections::HashMap, sync::Mutex},
    tokio::io::{self, AsyncRead, AsyncWrite},
};
pub struct ArchiveCache<S> {
    storage: S,
    // TODO: use an LRU or something useful. This is just a simple test of Caching + Archive.
    // TODO: use a RwLock here. Or ideally a lock-free data structure.
    cache: Mutex<HashMap<Addr, Vec<u8>>>,
}
#[async_trait::async_trait]
impl<S> CacheRead for ArchiveCache<S>
where
    S: StorageRead,
{
    type Structured = ArchivedStructured;
    async fn read_unstructured<A, W>(&self, addr: A, w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let addr = addr.as_ref();
        {
            let cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            if let Some(buf) = cache.get(addr) {
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
        StorageRead::read(self, addr, &mut buf).await?;
        let len = io::copy(&mut buf.as_slice(), &mut w).await?;
        let cache = self.cache.lock().map_err(|_| Error::Unhandled {
            message: "cache mutex poisoned".to_owned(),
        })?;
        if let Some(_) = cache.insert(addr.clone(), buf) {
            warn!("cache inserted twice, wasted storage read");
        }
        Ok(len)
    }
    async fn read_structured<A>(&self, addr: A) -> Result<&Self::Structured, Error>
    where
        A: AsRef<Addr> + 'static + Send,
    {
        todo!("read_struct")
    }
}
#[async_trait::async_trait]
impl<S> CacheWrite for ArchiveCache<S>
where
    S: StorageWrite,
{
    type Structured = Structured;
    async fn write_unstructured<R>(&self, r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        todo!("write")
    }
    async fn write_structured<T>(&self, structured: T) -> Result<Addr, Error>
    where
        T: Into<Self::Structured>,
    {
        todo!("write_struct")
    }
}
