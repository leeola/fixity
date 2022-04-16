//pub mod any_store;
pub mod json_store;
//pub mod rkyv_store;

use cid::{CidHasher, Hashers};

pub type Error = ();

pub mod cid {
    use multihash::MultihashDigest;

    const CID_LENGTH: usize = 34;
    pub type Cid = [u8; CID_LENGTH];

    pub trait CidHasher {
        type Cid;
        fn hash(&self, buf: &[u8]) -> Self::Cid;
        // A future fn to describe the underlying hasher.
        // Length, algo, etc.
        // fn desc() -> HasherDesc;
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Hashers {
        Blake3_256,
    }
    impl CidHasher for Hashers {
        type Cid = Cid;
        fn hash(&self, buf: &[u8]) -> Self::Cid {
            let hash = multihash::Code::from(*self).digest(&buf).to_bytes();
            match hash.try_into() {
                Ok(cid) => cid,
                Err(_) => {
                    // NIT:
                    unreachable!("multihash header + 256 fits into 34bytes")
                },
            }
        }
    }
    impl Default for Hashers {
        fn default() -> Self {
            Self::Blake3_256
        }
    }
    impl From<Hashers> for multihash::Code {
        fn from(h: Hashers) -> Self {
            // NIT: using the Multihash derive might make this a bit more idiomatic,
            // just not sure offhand if there's a way to do that while ensuring
            // we use the same codes as multihash.
            match h {
                Hashers::Blake3_256 => multihash::Code::Blake3_256,
            }
        }
    }
}

#[async_trait::async_trait]
pub trait Store<T, H = Hashers>
where
    H: CidHasher,
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
        // super::{any_store::AnyStore, json_store::JsonStore, rkyv_store::RkyvStore, *},
        super::{json_store::JsonStore, *},
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
    // #[case::test_any_store(AnyStore::new())]
    #[case::test_any_store(JsonStore::new())]
    // #[case::test_any_store(RkyvStore::new())]
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
