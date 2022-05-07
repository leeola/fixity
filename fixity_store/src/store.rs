// pub mod json_store;
// pub mod rkyv_store;

use {
    crate::{
        cid::{ContentHasher, ContentId, Hasher},
        deser::{Deserialize, SerdeJson, Serialize},
        storage::{memory::Memory, ContentStorage},
    },
    async_trait::async_trait,
    std::{marker::PhantomData, sync::Arc},
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
    type Cid: ContentId;
    type Hasher: ContentHasher<Self::Cid>;
    type Storage: ContentStorage<Self::Cid>;
    // async fn put<T>(&self, t: T) -> Result<Self::Cid, Error>
    // where
    //     T: Serialize + Send + 'static;
}

/*
pub struct Store<Storage, H = Hasher> {
    hasher: H,
    storage: Storage,
}
impl<S, H> Store<S, H>
where
    H: ContentHasher,
    S: ContentStorage<H::Cid>,
{
    pub fn new(storage: S) -> Self
    where
        H: Default,
    {
        Self {
            hasher: Default::default(),
            storage,
        }
    }
    pub async fn put<T>(&self, t: T) -> Result<H::Cid, Error>
    where
        T: Serialize + Send + 'static,
        S::Content: From<Vec<u8>>,
        H::Cid: Copy,
    {
        let buf = t.serialize().unwrap();
        let cid = self.hasher.hash(&buf);
        self.storage.write_unchecked(cid, buf).await?;
        Ok(cid)
    }
    async fn get<T>(&self, cid: &H::Cid) -> Result<ReprY<T>, Error>
    where
        T: Deserialize,
    {
        let buf = self.storage.read_unchecked(cid).await?;
        Ok(ReprY {
            buf: buf.into(),
            phantom_: PhantomData,
        })
    }
}
impl Store<Memory> {
    pub fn memory() -> Self {
        Self::new(Default::default())
    }
}
*/
pub struct ReprY<T> {
    buf: Arc<[u8]>,
    phantom_: PhantomData<T>,
}
impl<T> ReprY<T>
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
