// pub mod json_store;
// pub mod rkyv_store;

use {
    crate::{
        cid::{ContainedCids, ContentHasher, ContentId, Hasher, CID_LENGTH},
        deser::{Deserialize, SerdeJson, Serialize},
        storage::{self, ContentStorage},
    },
    async_trait::async_trait,
    std::{marker::PhantomData, ops::Deref, sync::Arc},
};

pub type Error = ();

pub struct Repo<Store> {
    store: Arc<Store>,
}
pub struct Branch<Store> {
    store: Arc<Store>,
}

#[async_trait]
pub trait Store: Send + Sync {
    type Deser: Send + Sync;
    type Cid: ContentId + 'static;
    type Hasher: ContentHasher<Self::Cid>;
    type Storage: ContentStorage<Self::Cid>;
    async fn put_with_cids<'a, T>(
        &self,
        t: &'a T,
        contained_cids: impl Iterator<Item = &'a Self::Cid> + Send + 'a,
    ) -> Result<Self::Cid, Error>
    where
        T: Serialize<Self::Deser> + Send + Sync;
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T>, Error>
    where
        T: Deserialize<Self::Deser>;
    async fn put<T>(&self, t: &T) -> Result<Self::Cid, Error>
    where
        T: Serialize<Self::Deser> + ContainedCids<Self::Cid> + Send + Sync,
    {
        let cids = t.contained_cids();
        self.put_with_cids(t, cids).await
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
    D: Send + Sync,
    H: ContentHasher<[u8; CID_LENGTH]>,
    S: ContentStorage<[u8; CID_LENGTH]>,
{
    type Deser = D;
    type Cid = [u8; CID_LENGTH];
    type Hasher = H;
    type Storage = S;

    async fn put_with_cids<'a, T>(
        &self,
        t: &'a T,
        _: impl Iterator<Item = &'a Self::Cid> + Send + 'a,
    ) -> Result<Self::Cid, Error>
    where
        T: Serialize<Self::Deser> + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = self.hasher.hash(buf.as_ref());
        self.storage.write_unchecked(cid, buf).await?;
        Ok(cid)
    }
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T>, Error>
    where
        T: Deserialize<Self::Deser>,
    {
        let buf = self.storage.read_unchecked(cid).await?;
        Ok(Repr {
            buf: buf.into(),
            phantom_: PhantomData,
        })
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
    async fn put_with_cids<'a, T>(
        &self,
        t: &'a T,
        contained_cids: impl Iterator<Item = &'a Self::Cid> + Send + 'a,
    ) -> Result<Self::Cid, Error>
    where
        T: Serialize<Self::Deser> + Send + Sync,
    {
        self.deref().put_with_cids(t, contained_cids).await
    }
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T>, Error>
    where
        T: Deserialize<Self::Deser>,
    {
        self.deref().get(cid).await
    }
}
pub struct Memory<D = SerdeJson, H = Hasher>(StoreImpl<storage::Memory, D, H>);
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
pub struct Repr<T> {
    buf: Arc<[u8]>,
    phantom_: PhantomData<T>,
}
impl<T> Repr<T>
where
    T: Deserialize,
{
    pub fn repr_to_owned(&self) -> Result<T, Error> {
        let value = T::deserialize_owned(self.buf.as_ref()).unwrap();
        Ok(value)
    }
    pub fn repr_ref(&self) -> Result<T::Ref<'_>, Error> {
        let value = T::deserialize_ref(self.buf.as_ref()).unwrap();
        Ok(value)
    }
}

#[cfg(test)]
pub mod test {
    use {super::*, crate::deser::DeserializeRef, rstest::*, std::fmt::Debug};
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
    impl DeserializeRef for Foo {
        type Ref<'a> = FooRef<'a>;
    }
    /*
    #[rstest]
    #[case::json(JsonStore::memory())]
    #[case::rkyv(RkyvStore::memory())]
    #[tokio::test]
    async fn store_poc<S>(#[case] store: S)
    where
        S: Store<String>,
        S: Store<Foo>,
        <<S as Store<String>>::Repr as Repr>::Borrow: Debug + PartialEq<str>,
        <<S as Store<Foo>>::Repr as Repr>::Borrow: Debug + PartialEq<Foo>,
    {
        let k1 = store.put(String::from("foo")).await.unwrap();
        let repr = Store::<String>::get(&store, &k1).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), String::from("foo"));
        assert_eq!(repr.repr_borrow().unwrap(), "foo");
        let k2 = store.put(Foo { name: "foo".into() }).await.unwrap();
        let repr = Store::<Foo>::get(&store, &k2).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), Foo { name: "foo".into() });
        assert_eq!(repr.repr_borrow().unwrap(), &Foo { name: "foo".into() });
        let k3 = store.put(String::from("bar")).await.unwrap();
        assert_eq!(
            Store::<String>::get(&store, &k1)
                .await
                .unwrap()
                .repr_borrow()
                .unwrap(),
            "foo"
        );
        assert_eq!(
            Store::<String>::get(&store, &k3)
                .await
                .unwrap()
                .repr_borrow()
                .unwrap(),
            "bar"
        );
    }
    */
    #[rstest]
    // #[case(Memory::<SerdeJson, Hasher>::new())]
    #[case(StoreImpl::<storage::Memory, SerdeJson, Hasher>::default())]
    #[tokio::test]
    async fn store_poc<S>(#[case] store: S)
    where
        S: Store,
    {
        let k1 = store.put(&String::from("foo")).await.unwrap();
        // let repr = store.get::<String>(&k1).await.unwrap();
        // assert_eq!(repr.repr_to_owned().unwrap(), String::from("foo"));
        // assert_eq!(repr.repr_ref().unwrap(), "foo");
        // let k2 = store.put(&Foo { name: "foo".into() }).await.unwrap();
        // let repr = store.get::<Foo>(&k2).await.unwrap();
        // assert_eq!(repr.repr_to_owned().unwrap(), Foo { name: "foo".into() });
        // assert_eq!(repr.repr_ref().unwrap(), FooRef { name: "foo" });
    }
}
