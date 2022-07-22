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
    type Bytes: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn exists(&self, cid: &Cid) -> Result<bool, ContentStoreError>;
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ContentStoreError>;
    async fn write_unchecked(&self, cid: &Cid, bytes: Vec<u8>) -> Result<(), ContentStoreError>;
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
    async fn write_unchecked(&self, cid: &Cid, bytes: Vec<u8>) -> Result<(), ContentStoreError> {
        self.deref().write_unchecked(cid, bytes).await
    }
}
