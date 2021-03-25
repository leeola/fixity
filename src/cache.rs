pub mod archive_cache;
pub use archive_cache::ArchiveCache;
use {
    crate::{
        primitive::{appendlog, commitlog, prollylist, prollytree},
        storage::{Error, StorageRead, StorageWrite},
        Addr,
    },
    std::convert::TryFrom,
    tokio::io::{AsyncRead, AsyncWrite},
};
/// An enum of all possible structured data stored within Fixity.
///
/// Unstructured data, aka raw bytes, are not represented within this
/// enum.
///
/// Primarily used with [`CacheRead`] and [`CacheWrite`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Structured {
    ProllyTreeNode(prollytree::NodeOwned),
    ProllyListNode(prollylist::NodeOwned),
    CommitLogNode(appendlog::LogNode<commitlog::CommitNode>),
}
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait CacheRead: Sync {
    /// The structured data that the cache can borrow and return in [`CacheRead::read_struct`].
    ///
    /// This is an associated type to allow implementers to define borrowed and alternate
    /// versions of the [`Structured`] data type. Generally this type should reflect [`Structured`],
    /// but doesn't have to be identical.
    type OwnedRef: OwnedRef;
    async fn read_unstructured<A, W>(&self, addr: A, w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send,
        W: AsyncWrite + Unpin + Send;
    async fn read_structured<A>(&self, addr: A) -> Result<Self::OwnedRef, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send;
}
pub trait OwnedRef {
    // Another lovely place to use GATs, whenever they hit stable, rather than returning a
    // &Self::Ref, implementers could return a Self::Ref<'a>, which would be neato.
    // type Ref<'a>;
    //
    // The lack of GATs mostly affects Serde-like low-cost deserialization, ie swapping
    // `Foo<String>` for `Foo<&'a str>`. So until then, Serde-like impls will
    // have to be less efficient and duplicate memory on first-read, even if using as_ref().
    type Ref;
    fn as_ref(&self) -> &Self::Ref;
    fn into_owned(self) -> Structured;
}
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait CacheWrite: Sync {
    async fn write_unstructured<R>(&self, r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send;
    async fn write_structured<T>(&self, structured: T) -> Result<Addr, Error>
    where
        T: Into<Structured> + Send;
}
/// A helper trait to allow a single `T` to return references to both a `Workspace` and
/// a `Cache`.
///
/// See [`Commit`](crate::Commit) for example usage.
pub trait AsCacheRef {
    type Cache: CacheRead + CacheWrite;
    fn as_cache_ref(&self) -> &Self::Cache;
}
impl From<prollytree::NodeOwned> for Structured {
    fn from(t: prollytree::NodeOwned) -> Self {
        Self::ProllyTreeNode(t)
    }
}
impl From<prollylist::NodeOwned> for Structured {
    fn from(t: prollylist::NodeOwned) -> Self {
        Self::ProllyListNode(t)
    }
}
impl TryFrom<Structured> for prollytree::NodeOwned {
    type Error = Error;
    fn try_from(t: Structured) -> Result<Self, Error> {
        dbg!(&t);
        match t {
            Structured::ProllyTreeNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
impl TryFrom<Structured> for prollylist::NodeOwned {
    type Error = Error;
    fn try_from(t: Structured) -> Result<Self, Error> {
        dbg!(&t);
        match t {
            Structured::ProllyListNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
impl TryFrom<Structured> for appendlog::LogNode<commitlog::CommitNode> {
    type Error = Error;
    fn try_from(t: Structured) -> Result<Self, Error> {
        dbg!(&t);
        match t {
            Structured::CommitLogNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
