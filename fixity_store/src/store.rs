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
// TODO: make send, maybe sync
pub trait Repr {
    type Owned;
    type Borrow;
    fn repr_to_owned(&self) -> Result<Self::Owned, Error>;
    fn repr_borrow(&self) -> Result<&Self::Borrow, Error>;
}

use {
    crate::deser::{Deserialize, Serialize},
    async_trait::async_trait,
};
#[async_trait]
pub trait StoreZ<H = Hasher>: Send
where
    H: ContentHasher,
{
    type Repr<T>: ReprZ<T>
    where
        T: Deserialize;
    async fn put<T>(&self, t: T) -> Result<H::Cid, Error>
    where
        T: Serialize + Send + 'static;
    async fn get<T>(&self, cid: &H::Cid) -> Result<Self::Repr<T>, Error>
    where
        T: Deserialize;
}
/*
#[async_trait]
pub trait Put<T, H = Hasher>: Send
where
    H: ContentHasher,
{
    async fn put_inner(&self, t: T) -> Result<H::Cid, Error>
    where
        T: Send + 'static;
}
#[async_trait]
pub trait Get<R, H = Hasher>: Send
where
    H: ContentHasher,
{
    async fn get(&self, cid: &H::Cid) -> Result<R, Error>;
}
*/
pub trait ReprZ<T>
where
    T: Deserialize,
{
    fn repr_into_owned(self) -> Result<T, Error>;
    fn repr_ref(&self) -> Result<T::Ref<'_>, Error>;
    // fn repr_borrow<'a, U: ?Sized>(&'a self) -> &'a U
    // where
    //     T::Ref<'a>: std::borrow::Borrow<U>,
    // {
    //     use std::borrow::Borrow;
    //     self.repr_ref().unwrap().borrow()
    // }
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
    // #[rstest]
    // #[case::json(JsonStore::memory())]
    // // #[case::rkyv(RkyvStore::memory())]
    // #[tokio::test]
    // async fn storez_poc<S>(#[case] store: S)
    // where
    //     S: StoreZ + Sync + Put<String>,
    // {
    //     let k1 = store.put(String::from("foo")).await.unwrap();
    // }
}
