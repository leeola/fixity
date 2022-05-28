pub mod memory;
pub use memory::Memory;
use {
    async_trait::async_trait,
    std::{str, sync::Arc},
};
type Error = ();
#[async_trait]
pub trait ContentStorage<Cid>: Send + Sync
where
    Cid: Send + Sync,
{
    type Content: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn exists(&self, cid: &Cid) -> Result<bool, Error>;
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Content, Error>;
    // TODO: Make this take a Into<Vec<u8>> + AsRef<[u8]>. Not gaining anything by requiring
    // the extra From<Vec<u8>> bound.
    async fn write_unchecked<B>(&self, cid: Cid, bytes: B) -> Result<(), Error>
    where
        B: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
}
#[async_trait]
pub trait MetaStorage<Cid>: Send + Sync
where
    Cid: Send + Sync,
{
    type Meta: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn read(&self, remote: &str, author: &Cid, prefix: &str) -> Result<Self::Meta, Error>;
    async fn write<B>(
        &self,
        remote: &str,
        author: &Cid,
        prefix: &str,
        bytes: B,
    ) -> Result<(), Error>
    where
        B: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
}
