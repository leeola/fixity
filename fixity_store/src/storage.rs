pub mod memory;
pub mod mut_storage;
use async_trait::async_trait;
pub use memory::Memory;
pub use mut_storage::MutStorage;
use std::{ops::Deref, str, sync::Arc};
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
impl<T, U, Cid> ContentStorage<Cid> for T
where
    Cid: Send + Sync + 'static,
    T: Deref<Target = U> + Send + Sync,
    U: ContentStorage<Cid>,
{
    type Content = U::Content;
    async fn exists(&self, cid: &Cid) -> Result<bool, Error> {
        self.deref().exists(cid).await
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Content, Error> {
        self.deref().read_unchecked(cid).await
    }
    async fn write_unchecked<B>(&self, cid: Cid, bytes: B) -> Result<(), Error>
    where
        B: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static,
    {
        self.deref().write_unchecked(cid, bytes).await
    }
}
