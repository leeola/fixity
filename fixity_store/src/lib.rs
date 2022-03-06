// A hopefully short term unstable feature, since GATs are stablizing soon.
#![feature(generic_associated_types)]
pub mod deser;

pub mod cid;
pub mod meta;
pub mod storage;
pub mod store;
pub mod content {
    use {
        crate::{
            deser::{Deserialize, Serialize},
            Store,
        },
        async_trait::async_trait,
    };
    pub type Error = ();
    #[async_trait]
    pub trait Content<S>: Sized + Send
    where
        S: Store,
    {
        async fn load(store: &S, cid: &S::Cid) -> Result<Self, Error>;
        async fn save(&self, store: &S) -> Result<S::Cid, Error>;
        async fn save_with_cids(&self, store: &S, cid_buf: &mut Vec<S::Cid>) -> Result<(), Error> {
            let cid = self.save(store).await?;
            cid_buf.push(cid);
            Ok(())
        }
    }
    #[async_trait]
    impl<T, S> Content<S> for T
    where
        S: Store,
        T: Serialize<S::Deser> + Deserialize<S::Deser> + Send + Sync,
    {
        async fn load(store: &S, cid: &S::Cid) -> Result<Self, Error> {
            let repr = store.get::<Self>(cid).await?;
            repr.repr_to_owned()
        }
        async fn save(&self, store: &S) -> Result<S::Cid, Error> {
            // store.put
            todo!()
        }
    }
}
pub use {
    cid::ContentHasher,
    meta::Meta,
    storage::{ContentStorage, MutStorage},
    store::Store,
};
pub type Error = ();

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
