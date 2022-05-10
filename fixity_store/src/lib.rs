// A hopefully short term unstable feature, since GATs are stablizing soon.
#![feature(generic_associated_types)]

pub mod cid;
pub mod storage;
pub mod store;
pub mod content {
    use {
        crate::cid::{ContentHasher, Hasher},
        async_trait::async_trait,
    };
    pub type Error = ();
    // #[async_trait]
    // pub trait Content<S, H = Hasher>: Sized + Send
    // where
    //     H: ContentHasher,
    // {
    //     async fn load(store: &S, cid: &H::Cid) -> Result<Self, Error>;
    //     async fn save(&self, store: &S) -> Result<H::Cid, Error>;
    //     async fn save_with_cids(&self, store: &S, cid_buf: &mut Vec<H::Cid>) -> Result<(), Error>
    //     where
    //         S: Sync,
    //     {
    //         let cid = self.save(store).await?;
    //         cid_buf.push(cid);
    //         Ok(())
    //     }
    // }
}
pub use cid::ContentHasher;
//    store::{Repr, Store},
pub type Error = ();

pub mod deser {
    pub use self::serde_json::SerdeJson;
    pub type Error = crate::Error;
    pub trait Serialize<Deser = SerdeJson> {
        fn serialize(&self) -> Result<Vec<u8>, Error>;
    }
    pub trait DeserializeRef<Deser = SerdeJson> {
        type Ref<'a>;
    }
    pub trait Deserialize<Deser = SerdeJson>: DeserializeRef<Deser> + Sized {
        fn deserialize_owned(buf: &[u8]) -> Result<Self, Error>;
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error>;
    }
    mod serde_json {
        use super::{Deserialize, DeserializeRef, Error, Serialize};
        pub struct SerdeJson;
        impl<T> Serialize<SerdeJson> for T
        where
            T: serde::Serialize,
        {
            fn serialize(&self) -> Result<Vec<u8>, Error> {
                let v = serde_json::to_vec(&self).unwrap();
                Ok(v)
            }
        }
        impl<T> Deserialize<SerdeJson> for T
        where
            T: DeserializeRef<SerdeJson> + serde::de::DeserializeOwned,
            for<'a> T::Ref<'a>: serde::Deserialize<'a>,
        {
            fn deserialize_owned(buf: &[u8]) -> Result<Self, Error> {
                let self_ = serde_json::from_slice(buf.as_ref()).unwrap();
                Ok(self_)
            }
            fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error> {
                let ref_ = serde_json::from_slice(buf.as_ref()).unwrap();
                Ok(ref_)
            }
        }
        // TODO: macro this.
        impl DeserializeRef<SerdeJson> for String {
            type Ref<'a> = &'a str;
        }
    }
    mod rkyv {
        pub struct Rkyv;
    }
}

/*
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
*/

pub trait FixityType {
    // fn serialize(&self, _??) -> Vec<u8>;
    fn generics(&self) -> &'static [&'static str];
    fn types(&self) -> &'static [&'static str];
}
// pub trait Deser: Sized {
//     fn de(t: &T) -> Vec<u8>;
// }
