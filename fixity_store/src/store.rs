use std::borrow::Borrow;

pub type Error = ();

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
//     type Ref<T>: StoreRef<T>;
//     async fn get<T>(&self) -> Self::Ref<T>;
// }

#[async_trait::async_trait]
pub trait Store<T> {
    type Key: AsRef<[u8]>;
    type Ref: StoreRef<T>;
    async fn put<K>(&self, t: T) -> Result<Self::Key, Error>
    where
        T: Send + 'static;
    async fn get<K>(&self, k: K) -> Result<Self::Ref, Error>
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
pub mod test_any_store {
    use {
        super::{Error, Store, StoreRef},
        std::{
            any::Any,
            borrow::Borrow,
            collections::HashMap,
            sync::{Arc, Mutex},
        },
    };
    pub struct AnyRef(Arc<dyn Any + Send + Sync>);
    pub struct TestAnyStore(Mutex<HashMap<Vec<u8>, AnyRef>>);
    impl TestAnyStore {
        pub fn new() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }
    #[async_trait::async_trait]
    impl<T> Store<T> for TestAnyStore
    where
        T: Any + Clone + 'static + Send + Sync,
    {
        type Key = usize;
        type Ref = AnyRef;
        async fn put<K>(&self, t: T) -> Result<(), ()> {
            self.0
                .lock()
                .unwrap()
                .insert(k.as_ref().to_vec(), AnyRef(Arc::new(t)));
            Ok(())
        }
        async fn get<K>(&self, k: K) -> Result<Self::Ref, ()>
        where
            K: AsRef<[u8]> + Send,
        {
            let t = {
                let map = self.0.lock().unwrap();
                AnyRef(Arc::clone(&map.get(k.as_ref()).unwrap().0))
            };
            Ok(t)
        }
    }
    impl<T> StoreRef<T> for AnyRef
    where
        T: Any + Clone,
    {
        type Repr = T;
        fn repr_to_owned(&self) -> Result<T, Error> {
            self.0.downcast_ref().map_or(Err(()), |t: &T| Ok(t.clone()))
        }
        fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
        where
            Self::Repr: Borrow<U>,
        {
            self.0
                .downcast_ref()
                .map_or(Err(()), |t: &T| Ok(t.borrow()))
        }
    }
}
/*
pub mod test_json_store {
    use {
        super::{Error, Store, StoreRef},
        std::{
            any::Any,
            borrow::Borrow,
            collections::HashMap,
            sync::{Arc, Mutex},
        },
    };
    struct JsonRef(Arc<Vec<u8>>);
    pub struct TestAnyStore(Mutex<HashMap<Vec<u8>, JsonRef>>);
    impl TestAnyStore {
        pub fn new() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }
    #[async_trait::async_trait]
    impl<T> Store<T> for TestAnyStore
    where
        T: Any + Clone + 'static + Send + Sync,
    {
        type Ref = JsonRef;
        async fn put<K>(&self, k: K, t: T) -> Result<(), ()>
        where
            K: AsRef<[u8]> + Send,
        {
            self.0
                .lock()
                .unwrap()
                .insert(k.as_ref().to_vec(), JsonRef(Arc::new(t)));
            Ok(())
        }
        async fn get<K>(&self, k: K) -> Result<Self::Ref, ()>
        where
            K: AsRef<[u8]> + Send,
        {
            let t = {
                let map = self.0.lock().unwrap();
                JsonRef(Arc::clone(&map.get(k.as_ref()).unwrap().0))
            };
            Ok(t)
        }
    }
    impl<T> StoreRef<T> for JsonRef
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
*/
#[cfg(test)]
pub mod test {
    use {
        super::{test_any_store::TestAnyStore, *},
        rstest::*,
    };
    #[rstest]
    #[case::test_any_store(TestAnyStore::new())]
    #[tokio::test]
    async fn store_poc<S>(#[case] store: S)
    where
        S: Store<String>,
        <<S as Store<String>>::Ref as StoreRef<String>>::Repr: Borrow<str>,
    {
        store.put(b"foo", String::from("foo")).await.unwrap();
        let ref_ = Store::<String>::get(&store, b"foo").await.unwrap();
        assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
        assert_eq!(ref_.repr_borrow::<str>().unwrap(), "foo");
    }
}
