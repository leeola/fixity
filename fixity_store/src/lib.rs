pub mod cid;
pub mod storage;
pub mod store;
pub mod content {
    use {
        crate::{
            cid::{ContentHasher, Hasher},
            store::Store,
        },
        async_trait::async_trait,
    };
    pub type Error = ();
    // FIXME: I hate the _and_ naming convention.. need something better.
    #[async_trait]
    pub trait Content<T, S, H = Hasher>: Sized + Send
    where
        S: Store<T, H>,
        H: ContentHasher,
    {
        async fn get(store: &S, cid: &H::Cid) -> Result<S::Repr, Error>;
        async fn put_and_head(&self, store: &S) -> Result<H::Cid, Error>;
        async fn put_and_all(&self, store: &S, cid_buf: &mut Vec<H::Cid>) -> Result<(), Error>
        where
            S: Sync,
        {
            let cid = self.put_and_head(store).await?;
            cid_buf.push(cid);
            Ok(())
        }
    }
}
pub mod prelude {
    pub use super::{cid::Hasher, Content, ContentHasher, Error, Store};
}
pub use {cid::ContentHasher, content::Content, store::Store};
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
