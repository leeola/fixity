use crate::{
    content_store::ContentStore,
    contentid::{Cid, NewContentId},
    deser::DeserError,
    store::StoreError,
};
use async_trait::async_trait;
use std::marker::PhantomData;

pub trait Serialize {
    // NIT: It would be nice if constructing an Arc<[u8]> from this was more clean.
    //
    // Also, keeping the AlignedVec from Rkyv would be really nice... this part of the API
    // is tough to keep compatible between Rkyv and Serde/etc.
    type Bytes: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
    fn serialize(&self) -> Result<Self::Bytes, DeserError>;
}
pub trait Deserialize: Sized {
    type Ref<'a>;
    fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError>;
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError>;
}

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
        let cid = <Cid as NewContentId>::hash(buf.as_ref());
        self.write_unchecked(&cid, buf.into()).await.unwrap();
        Ok(cid)
    }
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    where
        T: Serialize + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = <Cid as NewContentId>::hash(buf.as_ref());
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
#[cfg(feature = "rkyv")]
mod rkyv {
    use super::{Deserialize, Serialize};
    use crate::deser;
    use rkyv::{ser::serializers::AllocSerializer, AlignedVec, Infallible};

    impl<T> Serialize for T
    where
        T: rkyv::Archive + rkyv::Serialize<AllocSerializer<256>>,
        T::Archived: rkyv::Deserialize<T, Infallible>,
    {
        type Bytes = AlignedVec;
        fn serialize(&self) -> Result<Self::Bytes, deser::DeserError> {
            let aligned_vec = rkyv::to_bytes::<_, 256>(self).unwrap();
            Ok(aligned_vec)
        }
    }
    impl<T> Deserialize for T
    where
        T: rkyv::Archive,
        for<'a> <Self as rkyv::Archive>::Archived: rkyv::Deserialize<T, Infallible> + 'a,
    {
        type Ref<'a> = &'a <Self as rkyv::Archive>::Archived;
        fn deserialize_owned(buf: &[u8]) -> Result<Self, deser::DeserError> {
            crate::deser::rkyv::deserialize_owned::<Self>(buf)
        }
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, deser::DeserError> {
            crate::deser::rkyv::deserialize_ref::<Self>(buf)
        }
    }
}
