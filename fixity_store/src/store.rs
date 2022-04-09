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

const ADDR_LENGTH: usize = 34;
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
    fn repr_borrow<'a, U: ?Sized>(&'a self) -> Result<&'a U, Error>
    where
        Self::Repr: Borrow<U>;
}
pub trait ReprBorrow<'a, T>
where
    T: 'a,
{
    fn zrepr_borrow(&'a self) -> Result<T, Error>;
}
pub trait RefReprBorrow<T> {
    fn ref_repr_borrow<'a>(&'a self) -> Result<T, Error>
    where
        T: 'a;
}
pub trait PairRefReprBorrow<T> {
    fn pair_ref_repr_borrow<'a>(&'a self) -> Result<T, Error>
    where
        T: 'a;
}
// impl<'a, T, U> ReprBorrow<'a, &'a U> for T
// where
//     U: ?Sized,
//     T: Borrow<U>,
// {
//     fn repr_borrow(&'a self) -> Result<&'a U, Error> {
//         Ok(self.borrow())
//     }
// }
pub mod any_store {
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
    pub struct AnyStore<A = Addr>(Mutex<HashMap<A, DynRef>>);
    impl AnyStore {
        pub fn new() -> Self {
            Self(Mutex::new(HashMap::new()))
        }
    }
    #[async_trait::async_trait]
    impl<T, A> Store<T, A> for AnyStore<A>
    where
        T: Any + Clone + Send + Sync + 'static,
        A: From<Addr> + Clone + Hash + Eq + Send + Sync,
    {
        type Ref = AnyRef<T>;
        async fn put(&self, t: T) -> Result<A, ()> {
            let key = A::from([0u8; 34]);
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
    // impl<'a,  T, U> super::ReprBorrow<'a,  &'b U> for AnyRef<T>
    // where
    //     U: ?Sized,
    //     T: Borrow<U> + 'static,
    // {
    //     fn zrepr_borrow(&'a self) -> Result<&'b U, Error> {
    //         self.ref_
    //             .downcast_ref()
    //             .map_or(Err(()), |t: &T| Ok(t.borrow()))
    //     }
    // }
    // impl<'a, T, U> super::RefReprBorrow<&'a U> for AnyRef<T>
    // where
    //     U: ?Sized,
    //     T: Borrow<U> + 'static,
    // {
    //     fn ref_repr_borrow<'a>(&'a self) -> Result<&'a U, Error> {
    //         todo!()
    //         // self.ref_
    //         //     .downcast_ref()
    //         //     .map_or(Err(()), |t: &T| Ok(t.borrow()))
    //     }
    // }
    impl<T, U> super::PairRefReprBorrow<U> for AnyRef<T>
    where
        T: Borrow<U> + 'static,
    {
        fn pair_ref_repr_borrow<'a>(&'a self) -> Result<U, Error>
        where
            T: 'a,
        {
            todo!()
            // self.ref_
            //     .downcast_ref()
            //     .map_or(Err(()), |t: &T| Ok(t.borrow()))
        }
    }
}
/*
pub mod json_store {
    use {
        super::{Addr, Error, Store, StoreRef},
        multihash::MultihashDigest,
        serde::{de::DeserializeOwned, Deserialize, Serialize},
        std::{
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
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
        A: TryFrom<Vec<u8>> + Clone + Hash + Eq + Send + Sync,
    {
        type Ref = JsonRef<T>;
        async fn put(&self, t: T) -> Result<A, Error> {
            let buf: Vec<u8> = serde_json::to_vec(&t).unwrap();
            let addr: A = multihash::Code::Blake3_256
                .digest(&buf)
                .to_bytes()
                .try_into()
                .map_err(|_| ())?;
            self.0.lock().unwrap().insert(addr.clone(), Arc::from(buf));
            Ok(addr)
        }
        async fn get(&self, cid: &A) -> Result<Self::Ref, Error> {
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
        T: DeserializeOwned,
    {
        type Repr = T;
        fn repr_to_owned(&self) -> Result<T, Error> {
            use std::ops::Deref;
            let t: T = serde_json::from_slice(self.buf.deref()).unwrap();
            Ok(t)
        }
        fn repr_borrow<'a, U: ?Sized>(&'a self) -> Result<&'a U, Error>
        where
            Self::Repr: Borrow<U>,
            U: Deserialize<'a>,
        {
            todo!()
        }
    }
}
*/
#[cfg(test)]
pub mod test {
    use {
        super::{any_store::AnyStore, *},
        rstest::*,
    };
    // #[rstest]
    // #[case::test_any_store(AnyStore::new())]
    // // #[case::test_any_store(JsonStore::new())]
    // #[tokio::test]
    // async fn store_poc<'a, S>(#[case] store: S)
    // // async fn store_poc<'a, S>(store: S)
    // where
    //     S: Store<String>,
    //     <S as Store<String>>::Ref: PairRefReprBorrow<&'a str>,
    //     // <<S as Store<String>>::Ref as StoreRef<String>>::Repr: Borrow<str>,
    // {
    //     let k = store.put(String::from("foo")).await.unwrap();
    //     let ref_ = Store::<String>::get(&store, &k).await.unwrap();
    //     // let z = ref_.zrepr_borrow().unwrap();
    //     let z = ref_.pair_ref_repr_borrow().unwrap();
    //     assert_eq!(z, "fooz");
    //     // // assert_eq!(ref_.zrepr_borrow().unwrap(), "foo");
    //     // assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
    // }
    #[rstest]
    #[case::test_any_store(AnyStore::new())]
    // #[case::test_any_store(JsonStore::new())]
    #[tokio::test]
    async fn store_poc<'a, S>(#[case] store: S)
    // async fn store_poc<'a, S>(store: S)
    where
        S: Store<String>,
        <S as Store<String>>::Ref: PairRefReprBorrow<&'a str>,
        // <<S as Store<String>>::Ref as StoreRef<String>>::Repr: Borrow<str>,
    {
        let k = store.put(String::from("foo")).await.unwrap();
        let ref_ = Store::<String>::get(&store, &k).await.unwrap();
        // let z = ref_.zrepr_borrow().unwrap();
        let z = ref_.pair_ref_repr_borrow().unwrap();
        assert_eq!(z, "fooz");
        // // assert_eq!(ref_.zrepr_borrow().unwrap(), "foo");
        // assert_eq!(ref_.repr_to_owned().unwrap(), String::from("foo"));
    }
    // pub struct FooOwned {
    //     pub name: &',
    // }
    pub struct Foo<'a> {
        pub name: &'a str,
        pub handle: &'a str,
    }
}
