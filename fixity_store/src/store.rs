// pub mod json_store;
// pub mod rkyv_store;

use crate::{
    contentid::{Cid, ContentHasher, ContentId, Hasher, NewContentId, CID_LENGTH},
    deser::{Deserialize, Serialize},
    storage::{self, ContentStorage, StorageError},
};
use async_trait::async_trait;
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use thiserror::Error;

#[async_trait]
pub trait NewStore<Deser, Cid: NewContentId>: ContentStorage<Cid> {
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
pub trait Store: Send + Sync {
    type Deser: Send + Sync;
    type Cid: ContentId + 'static;
    type Hasher: ContentHasher<Self::Cid>;
    type Storage: ContentStorage<Self::Cid>;
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T, Self::Deser>, StoreError>
    where
        T: Deserialize<Self::Deser>;
    async fn put<T>(&self, t: &T) -> Result<Self::Cid, StoreError>
    where
        T: Serialize<Self::Deser> + Send + Sync;
    async fn put_with_cids<T>(
        &self,
        t: &T,
        cids_buf: &mut Vec<Self::Cid>,
    ) -> Result<(), StoreError>
    where
        T: Serialize<Self::Deser> + Send + Sync;
}
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("resource not found")]
    NotFound,
    #[error("resource not modified")]
    NotModified,
    #[error("storage: {0}")]
    Storage(StorageError),
}
impl From<StorageError> for StoreError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound => Self::NotFound,
            err => Self::Storage(err),
        }
    }
}
#[async_trait]
impl<S, U> Store for S
where
    S: Deref<Target = U> + Send + Sync,
    U: Store + Send + Sync,
{
    type Deser = U::Deser;
    type Cid = U::Cid;
    type Hasher = U::Hasher;
    type Storage = U::Storage;
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T, Self::Deser>, StoreError>
    where
        T: Deserialize<Self::Deser>,
    {
        self.deref().get(cid).await
    }
    async fn put<T>(&self, t: &T) -> Result<Self::Cid, StoreError>
    where
        T: Serialize<Self::Deser> + Send + Sync,
    {
        self.deref().put(t).await
    }
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Self::Cid>) -> Result<(), StoreError>
    where
        T: Serialize<Self::Deser> + Send + Sync,
    {
        self.deref().put_with_cids(t, cids_buf).await
    }
}
// NIT: Name sucks.
#[derive(Default)]
pub struct StoreImpl<Storage, Deser, Hasher> {
    hasher: Hasher,
    storage: Storage,
    _deser: PhantomData<Deser>,
}
impl<S, D, H> StoreImpl<S, D, H> {
    pub fn new(storage: S) -> Self
    where
        H: Default,
    {
        Self {
            hasher: Default::default(),
            storage,
            _deser: PhantomData,
        }
    }
}
#[async_trait]
impl<S, D, H> Store for StoreImpl<S, D, H>
where
    // FIXME: ... What? CID_LENGTH stopped working in the const Param here
    // as of beta-2022-09-20. Need to report this if it's still around
    // whenever i clean this code up.. something is fishy.
    S: ContentStorage<Cid<34>>,
    D: Send + Sync,
    H: ContentHasher<Cid<34>>,
{
    type Deser = D;
    type Cid = Cid<CID_LENGTH>;
    type Hasher = H;
    type Storage = S;

    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T, Self::Deser>, StoreError>
    where
        T: Deserialize<Self::Deser>,
    {
        let buf = self.storage.read_unchecked(cid).await?;
        Ok(Repr {
            buf: buf.into(),
            _t: PhantomData,
            _d: PhantomData,
        })
    }
    async fn put<T>(&self, t: &T) -> Result<Self::Cid, StoreError>
    where
        T: Serialize<Self::Deser> + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = self.hasher.hash(buf.as_ref());
        self.storage.write_unchecked(cid.clone(), buf).await?;
        Ok(cid)
    }
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Self::Cid>) -> Result<(), StoreError>
    where
        T: Serialize<Self::Deser> + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = self.hasher.hash(buf.as_ref());
        self.storage.write_unchecked(cid.clone(), buf).await?;
        cids_buf.push(cid);
        Ok(())
    }
}
pub struct Memory<D, H = Hasher>(StoreImpl<storage::Memory, D, H>);
impl<D, H> Memory<D, H> {
    pub fn new() -> Self
    where
        H: Default,
    {
        Self(StoreImpl::new(storage::Memory::default()))
    }
}
impl<D, H> Deref for Memory<D, H> {
    type Target = StoreImpl<storage::Memory, D, H>;
    fn deref(&self) -> &Self::Target {
        &self.0
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

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::deser::{DeserializeRef, Rkyv, SerdeJson};
    use rstest::*;
    use std::fmt::Debug;
    #[derive(
        Debug,
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    #[archive(compare(PartialEq))]
    #[archive_attr(derive(Debug))]
    pub struct Foo {
        pub name: String,
    }
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct FooRef<'a> {
        pub name: &'a str,
    }
    impl DeserializeRef<SerdeJson> for Foo {
        type Ref<'a> = FooRef<'a>;
    }
    #[rstest]
    #[case::memory(Memory::<SerdeJson, Hasher>::new())]
    #[case::memory_impl(StoreImpl::<storage::Memory, SerdeJson, Hasher>::default())]
    // #[case::impl_rkyv(StoreImpl::<storage::Memory, Rkyv, Hasher>::default())]
    #[tokio::test]
    async fn store_json<S>(#[case] store: S)
    where
        S: Store<Deser = SerdeJson>,
    {
        let k1 = store.put(&String::from("foo")).await.unwrap();
        let repr = store.get::<String>(&k1).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), String::from("foo"));
        assert_eq!(repr.repr_ref().unwrap(), "foo");
        let k2 = store.put(&Foo { name: "foo".into() }).await.unwrap();
        let repr = store.get::<Foo>(&k2).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), Foo { name: "foo".into() });
        assert_eq!(repr.repr_ref().unwrap(), FooRef { name: "foo" });
    }
    #[rstest]
    #[case::memory(Memory::<Rkyv, Hasher>::new())]
    #[case::memory_impl(StoreImpl::<storage::Memory, Rkyv, Hasher>::default())]
    // #[case::impl_rkyv(StoreImpl::<storage::Memory, Rkyv, Hasher>::default())]
    #[tokio::test]
    async fn store_rkyv<S>(#[case] store: S)
    where
        S: Store<Deser = Rkyv>,
    {
        let k1 = store.put(&String::from("foo")).await.unwrap();
        let repr = store.get::<String>(&k1).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), String::from("foo"));
        assert_eq!(repr.repr_ref().unwrap(), "foo");
        let k2 = store.put(&Foo { name: "foo".into() }).await.unwrap();
        let repr = store.get::<Foo>(&k2).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), Foo { name: "foo".into() });
        assert_eq!(repr.repr_ref().unwrap(), &Foo { name: "foo".into() });
    }
}
