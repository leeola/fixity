use crate::contentid::NewContentId;
use async_trait::async_trait;
use std::{ops::Deref, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteStoreError {
    #[error("resource not found")]
    NotFound,
    #[error("invalid input: {message}")]
    InvalidInput { message: String },
}
#[async_trait]
pub trait ByteStore<Cid: NewContentId>: Send + Sync {
    type Bytes: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn exists(&self, cid: &Cid) -> Result<bool, ByteStoreError>;
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ByteStoreError>;
    async fn write_unchecked(&self, cid: &Cid, bytes: Vec<u8>) -> Result<(), ByteStoreError>;
}
#[async_trait]
impl<T, Cid> ByteStore<Cid> for Arc<T>
where
    Cid: NewContentId,
    T: ByteStore<Cid>,
{
    type Bytes = T::Bytes;
    async fn exists(&self, cid: &Cid) -> Result<bool, ByteStoreError> {
        self.deref().exists(cid).await
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ByteStoreError> {
        self.deref().read_unchecked(cid).await
    }
    async fn write_unchecked(&self, cid: &Cid, bytes: Vec<u8>) -> Result<(), ByteStoreError> {
        self.deref().write_unchecked(cid, bytes).await
    }
}
