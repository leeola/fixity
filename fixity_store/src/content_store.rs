use crate::contentid::NewContentId;
use async_trait::async_trait;
use std::{ops::Deref, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContentStoreError {
    #[error("resource not found")]
    NotFound,
    #[error("invalid input: {message}")]
    InvalidInput { message: String },
}
#[async_trait]
pub trait ContentStore<Cid: NewContentId>: Send + Sync {
    // NIT: The conversion around the the generic byte types is .. annoying.
    // A single type (Into<Vec<u8>> for example) doesn't cover common cases.
    // So we either add a lot of conversions on the type, and hope they align..
    // or some types just end up needlessly converting. Which is unfortunate.
    //
    // Not sure the ideal solution.
    type Bytes: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn exists(&self, cid: &Cid) -> Result<bool, ContentStoreError>;
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ContentStoreError>;
    async fn write_unchecked<B>(&self, cid: &Cid, bytes: B) -> Result<(), ContentStoreError>
    where
        B: AsRef<[u8]> + Into<Arc<[u8]>> + Send;
    // TODO: Allow the caller to own the buf, for mutation of buf.
    // async fn read_unchecked_vec(&self, cid: &Cid) -> Result<Vec<u8>, ContentStoreError>;
}
#[async_trait]
impl<T, Cid> ContentStore<Cid> for Arc<T>
where
    Cid: NewContentId,
    T: ContentStore<Cid>,
{
    type Bytes = T::Bytes;
    async fn exists(&self, cid: &Cid) -> Result<bool, ContentStoreError> {
        self.deref().exists(cid).await
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ContentStoreError> {
        self.deref().read_unchecked(cid).await
    }
    async fn write_unchecked<B>(&self, cid: &Cid, bytes: B) -> Result<(), ContentStoreError>
    where
        B: AsRef<[u8]> + Into<Arc<[u8]>> + Send,
    {
        self.deref().write_unchecked(cid, bytes).await
    }
}
#[async_trait]
pub trait ContentStoreV2<Cid: NewContentId>: Send + Sync {
    type Bytes: AsRef<[u8]>;
    async fn exists(&self, cid: &Cid) -> Result<bool, ContentStoreError>;
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ContentStoreError>;
    /// Return the owned Vec of bytes for the given [`NewContentId`].
    async fn read_unchecked_vec(&self, cid: &Cid) -> Result<Vec<u8>, ContentStoreError>;
    async fn write_unchecked(&self, cid: &Cid, bytes: Vec<u8>) -> Result<(), ContentStoreError>
}
