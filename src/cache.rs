pub mod archive_cache;
use {
    crate::{
        primitive::{appendlog, commitlog, prollytree},
        storage::{Error, StorageRead, StorageWrite},
        Addr,
    },
    tokio::io::{AsyncRead, AsyncWrite},
};
/// An enum of all possible structured data stored within Fixity.
///
/// Unstructured data, aka raw bytes, are not represented within this
/// enum.
///
/// Primarily used with [`CacheRead`] and [`CacheWrite`].
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Structured {
    ProllyTreeNode(prollytree::NodeOwned),
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
        A: AsRef<Addr> + Send,
        W: AsyncWrite + Unpin + Send;
    async fn read_structured<A>(&self, addr: A) -> Result<Self::OwnedRef, Error>
    where
        A: AsRef<Addr> + Send;
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
