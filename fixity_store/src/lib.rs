pub mod cid;
pub mod store;

pub mod storage {
    use {
        crate::cid::{ContentHasher, Hashers},
        async_trait::async_trait,
        std::str,
    };
    type Error = ();
    #[async_trait]
    pub trait ContentStorage<V, H = Hashers>: Send + Sync
    where
        H: ContentHasher,
        V: AsRef<[u8]> + Send + 'static,
    {
        async fn exists(&self, cid: &H::Cid) -> Result<bool, Error>;
        async fn read(&self, cid: &H::Cid) -> Result<V, Error>;
        async fn write(&self, k: H::Cid, v: V) -> Result<(), Error>;
    }
    // NIT: Name TBD..?
    #[async_trait]
    pub trait ReflogStorage<H = Hashers>: Send + Sync
    where
        H: ContentHasher,
    {
        async fn exists<S>(&self, path: &[S]) -> Result<bool, Error>
        where
            S: AsRef<str> + Send + Sync;
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
