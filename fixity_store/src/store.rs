pub mod any_store;
pub mod json_store;
pub mod rkyv_store;

pub type Error = ();

const CID_LENGTH: usize = 34;
pub type Cid = [u8; CID_LENGTH];

#[async_trait::async_trait]
pub trait Store<T, C = Cid> {
    type Repr: Repr<Owned = T>;
    async fn put(&self, t: T) -> Result<C, Error>
    where
        T: Send + 'static,
        C: Send;
    async fn get(&self, k: &C) -> Result<Self::Repr, Error>
    where
        C: Send + Sync;
}
pub trait Repr {
    type Owned;
    type Borrow;
    fn repr_to_owned(&self) -> Result<Self::Owned, Error>;
    fn repr_borrow(&self) -> Result<&Self::Borrow, Error>;
}

#[cfg(test)]
pub mod test {
    use {
        super::{any_store::AnyStore, json_store::JsonStore, rkyv_store::RkyvStore, *},
        rstest::*,
        std::fmt::Debug,
    };
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
    #[rstest]
    #[case::test_any_store(AnyStore::new())]
    #[case::test_any_store(JsonStore::new())]
    #[case::test_any_store(RkyvStore::new())]
    #[tokio::test]
    async fn store_poc<'a, S>(#[case] store: S)
    where
        S: Store<Foo>,
        S: Store<String>,
        <<S as Store<String>>::Repr as Repr>::Borrow: AsRef<str>,
        <<S as Store<Foo>>::Repr as Repr>::Borrow: Debug + PartialEq<Foo>,
    {
        let k = store.put(String::from("foo")).await.unwrap();
        let repr = Store::<String>::get(&store, &k).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), String::from("foo"));
        assert_eq!(repr.repr_borrow().unwrap().as_ref(), "foo");
        let k = store.put(Foo { name: "foo".into() }).await.unwrap();
        let repr = Store::<Foo>::get(&store, &k).await.unwrap();
        assert_eq!(repr.repr_to_owned().unwrap(), Foo { name: "foo".into() });
        assert_eq!(repr.repr_borrow().unwrap(), &Foo { name: "foo".into() });
    }
}
