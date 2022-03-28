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

const ADDR_LENGTH: usize = 32;
pub type Addr = [u8; ADDR_LENGTH];

#[async_trait::async_trait]
pub trait Store<T, A = Addr> {
    type Ref: StoreRef<T>;
    async fn put(&self, t: T) -> Result<A, Error>
    where
        T: Send + 'static,
        A: Send;
    async fn get(&self, k: &A) -> Result<Self::Ref, Error>
    where
        A: Send + Sync;
}
pub trait StoreRef<T> {
    type Repr;
    fn repr_to_owned(&self) -> Result<T, Error>;
    fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
    where
        Self::Repr: Borrow<U>;
}
pub mod test_store {
    use {
        super::{Addr, Error, Store, StoreRef},
        std::{
            any::Any,
            borrow::Borrow,
            collections::HashMap,
            hash::Hash,
            marker::PhantomData,
            sync::{Arc, Mutex},
        },
    };
    pub struct AnyRef<T> {
        ref_: Arc<dyn Any + Send + Sync>,
        _phantom: PhantomData<T>,
    }
    type DynRef = Arc<dyn Any + Send + Sync>;
    pub struct TestStore<A = Addr>(Mutex<HashMap<A, DynRef>>);
    impl TestStore {
        pub fn new() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }
    #[async_trait::async_trait]
    impl<T, A> Store<T, A> for TestStore<A>
    where
        T: Any + Clone + Send + Sync + 'static,
        A: From<Addr> + Clone + Hash + Eq + Send + Sync,
    {
        type Ref = AnyRef<T>;
        async fn put(&self, t: T) -> Result<A, ()> {
            let key = A::from([0u8; 32]);
            self.0.lock().unwrap().insert(key.clone(), Arc::new(t));
            Ok(key)
        }
        async fn get(&self, cid: &A) -> Result<Self::Ref, ()> {
            let t = {
                let map = self.0.lock().unwrap();
                Arc::clone(&map.get(cid).unwrap())
            };
            Ok(AnyRef {
                ref_: t,
                _phantom: PhantomData,
            })
        }
    }
    impl<T> StoreRef<T> for AnyRef<T>
    where
        T: Any + Clone,
    {
        type Repr = T;
        fn repr_to_owned(&self) -> Result<T, Error> {
            self.ref_
                .downcast_ref()
                .map_or(Err(()), |t: &T| Ok(t.clone()))
        }
        fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
        where
            Self::Repr: Borrow<U>,
        {
            self.ref_
                .downcast_ref()
                .map_or(Err(()), |t: &T| Ok(t.borrow()))
        }
    }
}
pub mod json_store {
    use {
        super::{Addr, Error, Store, StoreRef},
        std::{
            any::Any,
            borrow::Borrow,
            collections::HashMap,
            hash::Hash,
            marker::PhantomData,
            sync::{Arc, Mutex},
        },
    };
    pub struct JsonRef<T> {
        buf: Arc<[u8]>,
        _phantom: PhantomData<T>,
    }
    // TODO: Back this store by an actual kv storage.
    pub struct JsonStore<A = Addr>(Mutex<HashMap<A, Arc<[u8]>>>);
    impl JsonStore {
        pub fn new() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }
    #[async_trait::async_trait]
    impl<T, A> Store<T, A> for JsonStore<A>
    where
        T: Any + Clone + Send + Sync + 'static,
        A: From<Addr> + Clone + Hash + Eq + Send + Sync,
    {
        type Ref = JsonRef<T>;
        async fn put(&self, t: T) -> Result<A, ()> {
            let addr = A::from([0u8; 32]);
            let buf: Vec<u8> = todo!("serialize T");
            self.0.lock().unwrap().insert(addr.clone(), Arc::from(buf));
            Ok(addr)
        }
        async fn get(&self, cid: &A) -> Result<Self::Ref, ()> {
            let buf = {
                let map = self.0.lock().unwrap();
                Arc::clone(&map.get(cid).unwrap())
            };
            Ok(JsonRef {
                buf,
                _phantom: PhantomData,
            })
        }
    }
    impl<T> StoreRef<T> for JsonRef<T>
    where
        T: Any + Clone,
    {
        type Repr = T;
        fn repr_to_owned(&self) -> Result<T, Error> {
            todo!()
        }
        fn repr_borrow<U: ?Sized>(&self) -> Result<&U, Error>
        where
            Self::Repr: Borrow<U>,
        {
            todo!()
        }
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::{json_store::JsonStore, test_store::TestStore, *},
        rstest::*,
    };
    #[rstest]
    #[case::test_any_store(TestStore::new())]
    #[case::test_any_store(JsonStore::new())]
    #[tokio::test]
    async fn store_poc<S>(#[case] store: S)
    where
        S: Store<String>,
        <<S as Store<String>>::Ref as StoreRef<String>>::Repr: Borrow<str>,
    {
        let k = store.put(String::from("foo")).await.unwrap();
        let ref_ = Store::<String>::get(&store, &k).await.unwrap();
        assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
        assert_eq!(ref_.repr_borrow::<str>().unwrap(), "foo");
    }
}
