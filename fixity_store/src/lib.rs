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
    #[async_trait]
    pub trait Content<S, H = Hasher>: Sized + Send
    where
        H: ContentHasher,
    {
        async fn load(store: &S, cid: &H::Cid) -> Result<Self, Error>;
        async fn save(&self, store: &S) -> Result<H::Cid, Error>;
        async fn save_with_cids(&self, store: &S, cid_buf: &mut Vec<H::Cid>) -> Result<(), Error>
        where
            S: Sync,
        {
            let cid = self.save(store).await?;
            cid_buf.push(cid);
            Ok(())
        }
    }
}
pub use {
    cid::ContentHasher,
    content::Content,
    store::{Repr, Store},
};
pub type Error = ();

pub mod deser {
    pub use self::serde_json::SerdeJson;
    pub trait Serialize<Deser = SerdeJson> {
        fn serialize(&self) -> Vec<u8>;
    }
    pub trait Deserialize<Deser = SerdeJson> {
        type Ref<'a>;
        fn deserialize_owned(buf: &[u8]) -> Self;
        fn deserialize_ref(buf: &[u8]) -> Self::Ref<'_>;
    }
    mod serde_json {
        use super::Serialize;
        pub struct SerdeJson;
        impl<T> Serialize<SerdeJson> for T
        where
            T: serde::Serialize,
        {
            fn serialize(&self) -> Vec<u8> {
                serde_json::to_vec(&self).unwrap()
            }
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
