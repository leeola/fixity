use crate::{
    content_store::ContentStore,
    contentid::NewContentId,
    deser::{DeserError, Serialize},
    store::StoreError,
};
use async_trait::async_trait;
use std::{marker::PhantomData, sync::Arc};

pub trait Deserialize: Sized {
    type Ref<'a>;
    fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError>;
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError>;
}

/// An extension trait for [`ContentStore`].
#[async_trait]
pub trait DeserExt<Cid: NewContentId>: ContentStore<Cid> {
    async fn get_unchecked<T>(&self, cid: &Cid) -> Result<DeserBuf<T>, StoreError>
    where
        T: Deserialize;
    async fn get_owned_unchecked<T>(&self, cid: &Cid) -> Result<T, StoreError>
    where
        T: Deserialize;
    async fn put<T>(&self, t: &T) -> Result<Cid, StoreError>
    where
        T: Serialize + Send + Sync;
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    where
        T: Serialize + Send + Sync;
}
#[derive(Clone, PartialEq, Eq)]
pub struct DeserBuf<B, T> {
    buf: B,
    _t: PhantomData<T>,
}
impl<B, T> DeserBuf<B, T>
where
    B: AsRef<[u8]>,
    T: Deserialize,
{
    pub fn buf_to_owned(&self) -> Result<T, StoreError> {
        let value = T::deserialize_owned(self.buf.as_ref()).unwrap();
        Ok(value)
    }
    pub fn buf_to_ref(&self) -> Result<T::Ref<'_>, StoreError> {
        let value = T::deserialize_ref(self.buf.as_ref()).unwrap();
        Ok(value)
    }
}
