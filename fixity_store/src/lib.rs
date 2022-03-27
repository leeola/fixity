#![feature(generic_associated_types)]

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    Addr,
    Uint32(u32),
    String(String),
}
pub enum Type {
    GCounter,
    CascadingCounter(Box<Type>),
}
pub enum Struct {
    GCounter(GCounter),
    LACounter(LACounter),
}
pub struct GCounter;
// Limited Alternate Counter
pub struct LACounter {
    limit: usize,
    inner: LACounterInner,
}
struct LACounterDeserializer;
struct LACounterInner {
    counter: GCounter,
    // alternate: Result<GCounter, Box<LACounterInner>>,
    alternate: Box<dyn Counter>,
}
pub trait Counter: FixityType {}
pub trait FixityType {
    // fn serialize(&self, _??) -> Vec<u8>;
    fn generics(&self) -> &'static [&'static str];
    fn types(&self) -> &'static [&'static str];
}
// pub trait Deser: Sized {
//     fn de(t: &T) -> Vec<u8>;
// }

// NIT: This is the type of trait i'd like to use to avoid so much
// TypeStore bubbling..
// assuming GATs allows for the impl to define additional
// constraints on T. Eg `impl Store where Self::Storer::T: Deserialize...`
//
// For now i'm leaving this impl as a future wish that might be impossible and
// perhaps makes no sense. :shrug:
//
// #[async_trait::async_trait]
// pub trait Store {
//     type Storer<T>: TypeStore<T>;
//     async fn of_type<T>(&self) -> Self::Storer<T>;
// }

use std::borrow::Borrow;
pub type Error = ();
#[async_trait::async_trait]
pub trait Store<T> {
    type Ref<RT>: StoreRef<RT>
    where
        RT: 'static;
    async fn put<K>(&self, k: K, t: T) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
        T: Send + 'static;
    async fn get<K>(&self, k: K) -> Result<Self::Ref<T>, Error>
    where
        K: AsRef<[u8]> + Send;
}
pub trait StoreRef<T> {
    type Repr;
    fn repr_to_owned(&self) -> Result<T, Error>;
    fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
    where
        Self::Repr: Borrow<U>;
}
pub mod mem_any_store {
    use {
        super::{Error, Store, StoreRef},
        std::{
            any::Any,
            borrow::Borrow,
            collections::HashMap,
            sync::{Arc, Mutex},
        },
    };
    type DynRef = Arc<dyn Any + Send + Sync>;
    pub struct TestStore(Mutex<HashMap<Vec<u8>, DynRef>>);
    impl TestStore {
        pub fn new() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }
    #[async_trait::async_trait]
    impl<T> Store<T> for TestStore
    where
        T: Any + Clone + 'static + Send + Sync,
        DynRef: Clone,
    {
        type Ref<T>
        where
            RT: 'static,
            RT: Clone,
        = DynRef;
        async fn put<K>(&self, k: K, t: T) -> Result<(), ()>
        where
            K: AsRef<[u8]> + Send,
        {
            self.0
                .lock()
                .unwrap()
                .insert(k.as_ref().to_vec(), Arc::new(t));
            Ok(())
        }
        async fn get<K>(&self, k: K) -> Result<Self::Ref<T>, ()>
        where
            K: AsRef<[u8]> + Send,
        {
            let t = {
                let map = self.0.lock().unwrap();
                Arc::clone(map.get(k.as_ref()).unwrap())
            };
            Ok(t)
        }
    }
    impl<T> StoreRef<T> for DynRef
    where
        T: Any + Clone,
    {
        type Repr = T;
        fn repr_to_owned(&self) -> Result<T, Error> {
            self.downcast_ref().map_or(Err(()), |t: &T| Ok(t.clone()))
        }
        fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
        where
            Self::Repr: Borrow<U>,
        {
            self.downcast_ref().map_or(Err(()), |t: &T| Ok(t.borrow()))
        }
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, rstest::*};
    /*
    #[rstest]
    #[case(mem_any_store::MemAnyStore::new())]
    #[tokio::test]
    // async fn store<S: TypeStore<String>>(#[case] store: S) {
    async fn store(#[case] store: mem_any_store::MemAnyStore) {
        store.put(b"foo", String::from("foo")).await.unwrap();
        assert_eq!(
            TypeStore::<String>::get(&store, b"foo")
                .await
                .unwrap()
                .to_owned_()
                .unwrap(),
            String::from("foo")
        );
        // assert_eq!(
        //     store.get(b"foo").await.unwrap().borrow::<str>().unwrap(),
        //     "zz"
        // );
    }
    */
    #[tokio::test]
    async fn foo() {
        let store = mem_any_store::TestStore::new();
        // let store = <mem_any_store::MemAnyStore as TypeStore::<String>>  =
        // mem_any_store::MemAnyStore::new();

        // bar(store).await;
        store.put(b"foo", String::from("foo")).await.unwrap();
        // let ref_ = <mem_any_store::MemAnyStore as TypeStore<String>>::get(&store, b"foo")
        //     .await
        //     .unwrap();
        let ref_ = Store::<String>::get(&store, b"foo").await.unwrap();
        // let s =
        //     <<mem_any_store::MemAnyStore as TypeStore<String>>::Ref as
        // TypeRef<String>>::to_owned_(         &ref_,
        //     )
        //     .unwrap();
        let s = StoreRef::<String>::repr_to_owned(&ref_).unwrap();
        assert_eq!(s, String::from("foo"));
        // <<mem_any_store::MemAnyStore as TypeStore<String>>::Ref as TypeRef<String>>::to_owned(
        assert_eq!(
            StoreRef::<String>::repr_to_owned(&Store::<String>::get(&store, b"foo").await.unwrap())
                .unwrap(),
            String::from("foo")
        );
        // assert_eq!(
        //     store.get(b"foo").await.unwrap().borrow::<str>().unwrap(),
        //     "zz"
        // );
    }
    async fn bar<S: Store<String>>(store: S) {
        store.put(b"foo", String::from("foo")).await.unwrap();
        assert_eq!(
            store.get(b"foo").await.unwrap().repr_to_owned().unwrap(),
            String::from("foo")
        );
        // assert_eq!(
        //     <S::Ref as TypeRef::<String>>::borrow::<str>(&store.get(b"foo").await.unwrap())
        //         .unwrap(),
        //     "zz"
        // );
    }
}
