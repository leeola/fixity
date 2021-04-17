use {
    crate::{
        core::{
            self,
            cache::{CacheRead, CacheWrite},
            workspace::Workspace,
            Commit,
        },
        Addr, Error,
    },
    tokio::io::{AsyncRead, AsyncWrite},
};
pub struct Bytes<'f>(Box<dyn InnerBytes + 'f>);
impl<'f> Bytes<'f> {
    pub fn new<C, W>(inner: core::Bytes<'f, C, W>) -> Self
    where
        C: CacheRead + CacheWrite,
        W: Workspace,
    {
        Self(Box::new(inner))
    }
    pub async fn read<Writer>(&self, mut w: Writer) -> Result<Option<u64>, Error>
    where
        Writer: AsyncWrite + Unpin + Send,
    {
        self.0.inner_read(&mut w).await
    }
    pub async fn write<R>(&self, mut r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        self.0.inner_write(&mut r).await
    }
    pub async fn commit(&self) -> Result<Addr, Error> {
        self.0.inner_commit().await
    }
}
#[async_trait::async_trait]
trait InnerBytes {
    async fn inner_read(
        &self,
        w: &mut (dyn AsyncWrite + Unpin + Send),
    ) -> Result<Option<u64>, Error>;
    async fn inner_write(&self, r: &mut (dyn AsyncRead + Unpin + Send)) -> Result<Addr, Error>;
    async fn inner_commit(&self) -> Result<Addr, Error>;
}
#[async_trait::async_trait]
impl<'f, C, W> InnerBytes for core::Bytes<'f, C, W>
where
    C: CacheRead + CacheWrite,
    W: Workspace,
{
    async fn inner_read(
        &self,
        w: &mut (dyn AsyncWrite + Unpin + Send),
    ) -> Result<Option<u64>, Error> {
        self.read(w).await
    }
    async fn inner_write(&self, r: &mut (dyn AsyncRead + Unpin + Send)) -> Result<Addr, Error> {
        self.write(r).await
    }
    async fn inner_commit(&self) -> Result<Addr, Error> {
        self.commit().await
    }
}
