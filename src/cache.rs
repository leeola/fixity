use {
    crate::{
        storage::{Error, StorageRead, StorageWrite},
        Addr,
    },
    tokio::io::AsyncRead,
};
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait CacheRead: Sync {
    type Buf: AsRef<[u8]>;
    async fn read<A>(&self, addr: A) -> Result<Self::Buf, Error>
    where
        A: AsRef<Addr> + 'static + Send;
}
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait CacheWrite: Sync {
    async fn write<A, R>(&self, addr: A, r: R) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        R: AsyncRead + Unpin + Send;
}
/// A helper trait to allow a single `T` to return references to both a `Workspace` and
/// a `Cache`.
///
/// See [`Commit`](crate::Commit) for example usage.
pub trait AsCacheRef {
    type Cache: CacheRead + CacheWrite;
    fn as_cache_ref(&self) -> &Self::Cache;
}

#[async_trait::async_trait]
impl<T> CacheRead for T
where
    T: StorageRead,
{
    type Buf = Vec<u8>;
    async fn read<A>(&self, addr: A) -> Result<Self::Buf, Error>
    where
        A: AsRef<Addr> + 'static + Send,
    {
        let mut buf = Vec::new();
        StorageRead::read(self, addr, &mut buf).await?;
        Ok(buf)
    }
}
#[async_trait::async_trait]
impl<T> CacheWrite for T
where
    T: StorageWrite,
{
    async fn write<A, R>(&self, addr: A, r: R) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        R: AsyncRead + Unpin + Send,
    {
        StorageWrite::write(self, addr, r).await
    }
}
