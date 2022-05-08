// pub mod json_store;
// pub mod rkyv_store;

use {
    crate::{
        cid::{ContainedCids, ContentHasher, ContentId, Hasher, CID_LENGTH},
        deser::{Deserialize, SerdeJson, Serialize},
        storage::{memory::Memory, ContentStorage},
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
pub trait Store {
    type Cid: ContentId + 'static;
    type Hasher: ContentHasher<Self::Cid>;
    type Storage: ContentStorage<Self::Cid>;
    async fn put_with_cids<'a, T, D>(
        &self,
        t: &'a T,
        contained_cids: impl Iterator<Item = &'a Self::Cid> + Send + 'a,
    ) -> Result<Self::Cid, Error>
    where
        T: Serialize<D> + Send + Sync;
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T>, Error>
    where
        T: Deserialize;
    async fn put<T, D>(&self, t: &T) -> Result<Self::Cid, Error>
    where
        T: Serialize<D> + ContainedCids<Self::Cid> + Send + Sync,
    {
        let cids = t.contained_cids();
        self.put_with_cids(t, cids).await
    }
}

// NIT: Name sucks.
pub struct StoreImpl<Storage, Hasher> {
    hasher: Hasher,
    storage: Storage,
}
impl<S, H> StoreImpl<S, H> {
    pub fn new(storage: S) -> Self
    where
        H: Default,
    {
        Self {
            hasher: Default::default(),
            storage,
        }
    }
}
#[async_trait]
impl<S, H> Store for StoreImpl<S, H>
where
    H: ContentHasher<[u8; CID_LENGTH]>,
    S: ContentStorage<[u8; CID_LENGTH]>,
{
    type Cid = [u8; CID_LENGTH];
    type Hasher = H;
    type Storage = S;

    async fn put_with_cids<'a, T, D>(
        &self,
        t: &'a T,
        _: impl Iterator<Item = &'a Self::Cid> + Send + 'a,
    ) -> Result<Self::Cid, Error>
    where
        T: Serialize<D> + Send + Sync,
    {
        let buf = t.serialize().unwrap();
        let cid = self.hasher.hash(&buf);
        self.storage.write_unchecked(cid, buf).await?;
        Ok(cid)
    }
    async fn get<T>(&self, cid: &Self::Cid) -> Result<Repr<T>, Error>
    where
        T: Deserialize,
    {
        let buf = self.storage.read_unchecked(cid).await?;
        Ok(Repr {
            buf: buf.into(),
            phantom_: PhantomData,
        })
    }
}
// impl Store<Memory> {
//     pub fn memory() -> Self {
//         Self::new(Default::default())
//     }
// }
pub struct Repr<T> {
    buf: Arc<[u8]>,
    phantom_: PhantomData<T>,
}
impl<T> Repr<T>
where
    T: Deserialize,
{
    pub fn repr_into_owned(self) -> Result<T, Error> {
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
    use {super::*, rstest::*, std::fmt::Debug};
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
    // #[rstest]
    // #[case(Store::memory())]
    // #[tokio::test]
    // async fn storey_poc<S, H>(#[case] store: Store<S, H>)
    // where
    //     H: ContentHasher,
    //     S: ContentStorage<H::Cid>,
    // {
    //     //let k1 = store.put(String::from("foo")).await.unwrap();
    // }
}
