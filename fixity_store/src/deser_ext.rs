use crate::{
    content_store::ContentStore,
    contentid::{Cid, ContentId},
    deser::{Deserialize, Serialize},
    store::StoreError,
};
use async_trait::async_trait;
use std::marker::PhantomData;

/// An rait for [`ContentStore`], [de]serializing content as needed.
#[async_trait]
pub trait DeserExt: ContentStore {
    async fn get_unchecked<T>(&self, cid: &Cid) -> Result<DeserBuf<Self::Bytes, T>, StoreError>
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
#[async_trait]
impl<S> DeserExt for S
where
    S: ContentStore,
{
    async fn get_unchecked<T>(&self, cid: &Cid) -> Result<DeserBuf<Self::Bytes, T>, StoreError>
    where
        T: Deserialize,
    {
        let buf = self.read_unchecked(cid).await.unwrap();
        Ok(DeserBuf {
            buf,
            _t: PhantomData,
        })
    }
    async fn get_owned_unchecked<T>(&self, cid: &Cid) -> Result<T, StoreError>
    where
        T: Deserialize,
    {
        let buf = self.read_unchecked(cid).await.unwrap();
        DeserBuf {
            buf,
            _t: PhantomData,
        }
        .buf_to_owned()
    }
    async fn put<T>(&self, t: &T) -> Result<Cid, StoreError>
    where
        T: Serialize + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = <Cid as ContentId>::hash(buf.as_ref());
        self.write_unchecked(&cid, buf.into()).await.unwrap();
        Ok(cid)
    }
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    where
        T: Serialize + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = <Cid as ContentId>::hash(buf.as_ref());
        self.write_unchecked(&cid, buf.into()).await.unwrap();
        cids_buf.push(cid);
        Ok(())
    }
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
