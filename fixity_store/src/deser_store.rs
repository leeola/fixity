use std::{marker::PhantomData, sync::Arc};

use crate::{
    byte_store::ByteStore,
    contentid::NewContentId,
    deser::{Deserialize, Serialize},
    store::StoreError,
};
use async_trait::async_trait;

#[async_trait]
pub trait DeserStore<Deser, Cid: NewContentId>: ByteStore<Cid> {
    async fn get<T>(&self, cid: &Cid) -> Result<Repr<T, Deser>, StoreError>
    where
        T: Deserialize<Deser>;
    async fn put<T>(&self, t: &T) -> Result<Cid, StoreError>
    where
        T: Serialize<Deser> + Send + Sync;
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    where
        T: Serialize<Deser> + Send + Sync;
}
#[async_trait]
impl<Deser, Cid, U> DeserStore<Deser, Cid> for U
where
    Cid: NewContentId,
    U: ByteStore<Cid>,
{
    async fn get<T>(&self, cid: &Cid) -> Result<Repr<T, Deser>, StoreError>
    where
        T: Deserialize<Deser>,
    {
        let buf = self.read_unchecked(cid).await.unwrap();
        Ok(Repr {
            buf: buf.into(),
            _t: PhantomData,
            _d: PhantomData,
        })
    }

    async fn put<T>(&self, t: &T) -> Result<Cid, StoreError>
    where
        T: Serialize<Deser> + Send + Sync,
    {
        todo!()
    }

    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    where
        T: Serialize<Deser> + Send + Sync,
    {
        todo!()
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Repr<T, D> {
    buf: Arc<[u8]>,
    _t: PhantomData<T>,
    _d: PhantomData<D>,
}
impl<T, D> Repr<T, D>
where
    T: Deserialize<D>,
{
    pub fn repr_to_owned(&self) -> Result<T, StoreError> {
        let value = T::deserialize_owned(self.buf.as_ref()).unwrap();
        Ok(value)
    }
    pub fn repr_ref(&self) -> Result<T::Ref<'_>, StoreError> {
        let value = T::deserialize_ref(self.buf.as_ref()).unwrap();
        Ok(value)
    }
}
