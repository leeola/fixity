// A hopefully short term unstable feature, stype Container = ype Container = nce GATs are
// stablizing soon.
pub mod container;
pub mod contentid;
pub mod deser;
pub mod deser_ext;
pub mod replicaid;
pub mod storage;
pub mod store;
pub use storage::{ContentStorage, MutStorage};
pub mod container_store;
pub mod content_store;
pub mod meta_store;
pub mod mut_store;
pub mod stores;
pub mod prelude {
    pub use crate::{
        content_store::ContentStore,
        deser::{Deserialize, Serialize},
        deser_ext::DeserExt,
        meta_store::MetaStore,
    };
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
