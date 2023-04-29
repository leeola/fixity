use crate::{
    content_store::{ContentStore, ContentStoreError},
    contentid::{Cid, NewContentId},
    deser::DeserError,
    store::StoreError,
};
use async_trait::async_trait;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::Arc,
};

pub struct Owned;

pub trait DeserializeRef<Repr>: Sized {
    type Ref<'a>;
}

#[async_trait]
pub trait DeserStoreV3<Cid: NewContentId>: ContentStore {
    type DeserRepr<T, Repr>: Deserialize<T, Repr>
    where
        T: DeserializeRef<Repr>;
    async fn get<T, Repr>(&self, cid: &Cid) -> Result<Self::DeserRepr<T, Repr>, StoreError>
    where
        T: DeserializeRef<Repr>,
        Self::DeserRepr<T, Repr>: Deserialize<T, Repr>;
    // async fn put<T>(&self, t: &T) -> Result<Cid, StoreError>
    // where
    //     Self: Serialize<T>,
    //     T: Send + Sync;
    // async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    // where
    //     Self: Serialize<T>,
    //     T: Send + Sync;
}
// NIT: I wanted to remove the generic params such that any implementation of `Deserialize` would
// only have a single output type to further remove ambiguity. However i was getting type bound
// overflows and so i backed off that decision. Maybe it can be revisited in the future, but
// hopefully it's mostly unnecessary.
pub trait Deserialize<T, Repr>
where
    T: DeserializeRef<Repr>,
{
    fn deserialize_owned(buf: &[u8]) -> Result<T, DeserError>;
    fn deserialize_ref(buf: &[u8]) -> Result<T::Ref<'_>, DeserError>;
}
pub trait Serialize<T> {
    fn serialize(&self, t: &T) -> Result<Vec<u8>, DeserError>;
}

pub struct DeserStoreImpl<Deser, Store> {
    _d: PhantomData<Deser>,
    store: Store,
}
impl<D, S> From<S> for DeserStoreImpl<D, S> {
    fn from(store: S) -> Self {
        Self {
            _d: PhantomData,
            store,
        }
    }
}
#[async_trait]
impl<D, S> ContentStore for DeserStoreImpl<D, S>
where
    S: ContentStore,
    D: Send + Sync,
{
    type Bytes = S::Bytes;
    async fn exists(&self, cid: &Cid) -> Result<bool, ContentStoreError> {
        self.store.exists(cid).await
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ContentStoreError> {
        self.store.read_unchecked(cid).await
    }
    async fn write_unchecked<B>(&self, cid: &Cid, bytes: B) -> Result<(), ContentStoreError>
    where
        B: AsRef<[u8]> + Into<Arc<[u8]>> + Send,
    {
        self.store.write_unchecked(cid, bytes).await
    }
}
#[async_trait]
impl<S, Cid> DeserStoreV3<Cid> for DeserStoreImpl<DeserJson, S>
where
    Cid: NewContentId,
    S: ContentStore,
{
    type DeserRepr<T: DeserializeRef<Repr>, Repr> = DeserReprImpl<DeserJson, T, Repr>;
    // type DeserRepr<T: DeserializeRef<Repr>, Repr> = DeserReprImpl<D, T, Repr>;
    async fn get<T, Repr>(&self, cid: &Cid) -> Result<Self::DeserRepr<T, Repr>, StoreError>
    where
        T: DeserializeRef<Repr>,
        Self::DeserRepr<T, Repr>: Deserialize<T, Repr>,
    {
        todo!()
    }
}
pub struct DeserJson;
#[derive(Clone, PartialEq, Eq)]
pub struct DeserReprImpl<Deser, T, Repr> {
    buf: Arc<[u8]>,
    _d: PhantomData<Deser>,
    _t: PhantomData<T>,
    _r: PhantomData<Repr>,
}
impl<T, R> Deserialize<T, R> for DeserReprImpl<DeserJson, T, R>
where
    T: DeserializeRef<R>,
{
    fn deserialize_owned(buf: &[u8]) -> Result<T, DeserError> {
        todo!()
    }
    fn deserialize_ref(buf: &[u8]) -> Result<T::Ref<'_>, DeserError> {
        todo!()
    }
}
