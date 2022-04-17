pub mod json_store;
pub mod rkyv_store;

use crate::cid::{ContentHasher, Hasher};

pub type Error = ();

#[async_trait::async_trait]
pub trait Store<T, H = Hasher>: Send
where
    H: ContentHasher,
{
    type Repr: Repr<Owned = T>;
    async fn put(&self, t: T) -> Result<H::Cid, Error>
    where
        T: Send + 'static;
    async fn get(&self, k: &H::Cid) -> Result<Self::Repr, Error>;
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
        super::{json_store::JsonStore, rkyv_store::RkyvStore, *},
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
    #[case::json(JsonStore::memory())]
    #[case::rkyv(RkyvStore::memory())]
    #[tokio::test]
    async fn store_poc<'a, S>(#[case] store: S)
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
}
