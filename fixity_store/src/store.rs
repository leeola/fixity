// pub mod any_store;
pub mod json_store;
// pub mod rkyv_store;

pub type Error = ();

const ADDR_LENGTH: usize = 34;
pub type Addr = [u8; ADDR_LENGTH];

#[async_trait::async_trait]
pub trait Store<T, C = Addr> {
    type Repr: Repr<T>;
    async fn put(&self, t: T) -> Result<C, Error>
    where
        T: Send + 'static,
        C: Send;
    async fn get(&self, k: &C) -> Result<Self::Repr, Error>
    where
        C: Send + Sync;
}
pub trait Repr<T> {
    fn repr_to_owned(&self) -> Result<T, Error>;
    fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
    where
        Self: ReprBorrow<U>,
    {
        ReprBorrow::borrow_from_repr(self)
    }
}
pub trait ReprBorrow<Borrowed: ?Sized> {
    fn borrow_from_repr(&self) -> Result<&Borrowed, Error>;
}
#[cfg(test)]
pub mod test {
    use {
        super::{
            //any_store::AnyStore,
            json_store::JsonStore,
            // rkyv_store::RkyvStore,
            *,
        },
        rkyv::string::ArchivedString,
        rstest::*,
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
    // #[rstest]
    // #[case::test_any_store(RkyvStore::new())]
    // #[tokio::test]
    // async fn rkyv_store_poc<'a, S>(#[case] store: S)
    // where
    //     S: Store<String> + Store<Foo>,
    //     <S as Store<String>>::Repr: ReprBorrow<ArchivedString>,
    //     <S as Store<Foo>>::Repr: ReprBorrow<ArchivedFoo>,
    // {
    //     let k = store.put(String::from("foo")).await.unwrap();
    //     let ref_ = Store::<String>::get(&store, &k).await.unwrap();
    //     assert_eq!(ref_.repr_borrow().unwrap(), "foo");
    //     assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
    //     let k = store.put(Foo { name: "foo".into() }).await.unwrap();
    //     let ref_ = Store::<Foo>::get(&store, &k).await.unwrap();
    //     assert_eq!(ref_.repr_borrow().unwrap(), &Foo { name: "foo".into() });
    //     assert_eq!(ref_.repr_to_owned().unwrap(), Foo { name: "foo".into() });
    // }
    // #[rstest]
    // #[case::test_any_store(AnyStore::new())]
    // // #[case::test_any_store(JsonStore::new())]
    // #[tokio::test]
    // async fn store_poc<'a, S>(#[case] store: S)
    // where
    //     S: Store<String> + Store<Foo>,
    //     <S as Store<String>>::Repr: ReprBorrow<str>,
    //     <S as Store<Foo>>::Repr: ReprBorrow<Foo>,
    // {
    //     let k = store.put(String::from("foo")).await.unwrap();
    //     let ref_ = Store::<String>::get(&store, &k).await.unwrap();
    //     assert_eq!(ref_.repr_borrow().unwrap(), "foo");
    //     assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
    //     let k = store.put(Foo { name: "foo".into() }).await.unwrap();
    //     let ref_ = Store::<Foo>::get(&store, &k).await.unwrap();
    //     assert_eq!(ref_.repr_borrow().unwrap(), &Foo { name: "foo".into() });
    //     assert_eq!(ref_.repr_to_owned().unwrap(), Foo { name: "foo".into() });
    // }
    pub struct FooPartialCopy<'a> {
        pub name: &'a str,
    }
    #[rstest]
    #[case::test_any_store(JsonStore::new())]
    #[tokio::test]
    async fn json_poc(#[case] store: JsonStore)
    // async fn json_poc<'a, S>(#[case] store: S)
    // where
    //     S: Store<String> + Store<Foo>,
    //     <S as Store<String>>::Repr: ReprBorrow<str>,
    //     <S as Store<Foo>>::Repr: ReprBorrow<Foo>,
    {
        use json_store::{ReprBorrowRef, ReprBorrowRefBlah};
        let k = store.put(String::from("foo")).await.unwrap();
        let ref_ = Store::<String>::get(&store, &k).await.unwrap();
        assert_eq!(ref_.ref_from_repr::<&str>().unwrap(), "fooz");
        assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
        // let k = store.put(Foo { name: "foo".into() }).await.unwrap();
        // let ref_ = Store::<Foo>::get(&store, &k).await.unwrap();
        // assert_eq!(ref_.repr_borrow().unwrap(), &Foo { name: "foo".into() });
        // assert_eq!(ref_.repr_to_owned().unwrap(), Foo { name: "foo".into() });
    }
}
