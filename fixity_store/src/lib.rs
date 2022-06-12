// A hopefully short term unstable feature, since GATs are stablizing soon.
#![feature(generic_associated_types)]
pub mod deser;

pub mod cid;
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

pub mod meta {
    use {super::Error, async_trait::async_trait};

    #[async_trait]
    pub trait Meta<Rid, Cid>: Send + Sync
    where
        Rid: Send + Sync,
        Cid: Send + Sync,
    {
        async fn repos(&self, remote: &str) -> Result<Vec<String>, Error>;
        async fn branches(&self, remote: &str, repo: &str) -> Result<Vec<String>, Error>;
        async fn heads(
            &self,
            remote: &str,
            repo: &str,
            branch: &str,
        ) -> Result<Vec<(Rid, Cid)>, Error>;
        async fn head(
            &self,
            remote: &str,
            repo: &str,
            branch: &str,
            replica: &Rid,
        ) -> Result<Cid, Error>;
        async fn set_head(
            &self,
            remote: &str,
            repo: &str,
            replica: Rid,
            head: Cid,
        ) -> Result<(), Error>;
        async fn detatch_head(
            &self,
            remote: &str,
            repo: &str,
            replica: Rid,
            head: Cid,
        ) -> Result<(), Error>;
        async fn append_log(
            &self,
            remote: &str,
            repo: &str,
            replica: Rid,
            head: Cid,
            message: &str,
        ) -> Result<(), Error>;
        async fn logs(
            &self,
            remote: &str,
            repo: &str,
            replica: Rid,
            offset: usize,
            limit: usize,
        ) -> Result<Vec<Log<Rid, Cid>>, Error>;
    }
    #[derive(Debug)]
    pub struct Log<Rid, Cid> {
        pub remote: String,
        pub repo: String,
        pub replica: Rid,
        pub head: Cid,
        pub message: String,
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
