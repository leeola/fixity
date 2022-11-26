use crate::{
    contentid::{ContentId, NewContentId},
    type_desc::TypeDescription,
};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};
use thiserror::Error;

pub trait NewReplicaId:
    Clone + Sized + Send + Sync + Eq + Ord + Hash + Debug + Display + 'static + TypeDescription
{
    type Buf: AsRef<[u8]>;
    /// Generate a new `ReplicaId`.
    //
    // TODO: allow for configured randomness. Perhaps taking a `rand` value as a param?
    fn new() -> Self;
    /// Construct a replica identifier from the given buffer.
    fn from_buf(hash: Vec<u8>) -> Result<Self, FromBufError>;
    fn as_buf(&self) -> &Self::Buf;
    fn len(&self) -> usize {
        self.as_buf().as_ref().len()
    }
}
#[derive(Error, Debug)]
pub enum FromBufError {
    #[error("invalid length")]
    Length,
}

// TODO: Remove bounds, impl methods manually - so ReplicaId doesn't impl ContentId,
// since they have no direct relation.
pub trait ReplicaId: ContentId {}
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// TODO: Serde doesn't impl for const :(. Can i impl manually perhaps?
// #[cfg(feature = "serde")]
// #[derive(serde::Deserialize, serde::Serialize)]
#[cfg(feature = "rkyv")]
#[derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
#[cfg(feature = "rkyv")]
#[archive(compare(PartialEq, PartialOrd))]
// #[cfg(feature = "rkyv")]
// #[archive_attr(derive(From))]
pub struct Rid<const N: usize>([u8; N]);
impl<const N: usize> ReplicaId for Rid<N> {}
impl<const N: usize> ContentId for Rid<N> {
    fn from_hash(hash: Vec<u8>) -> Option<Self> {
        <[u8; N]>::try_from(hash).ok().map(Self)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<const N: usize> Default for Rid<N>
where
    [u8; N]: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<const N: usize> Debug for Rid<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // PERF: Can we fork multibase to make a non-allocating display? I would think
        // yes offhand, so i think this Display is okay for now - hoping that in the nearish
        // future we can provide an alt impl of encode that writes chars to the formatter
        // directly.
        write!(f, "Rid<{}>({})", self.0.len(), self.encode())
    }
}
impl<const N: usize> Display for Rid<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // PERF: Can we fork multibase to make a non-allocating display? I would think
        // yes offhand, so i think this Display is okay for now - hoping that in the nearish
        // future we can provide an alt impl of encode that writes chars to the formatter
        // directly.
        write!(f, "{}", self.encode())
    }
}
impl<const N: usize> AsRef<[u8]> for Rid<N> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl<const N: usize> From<[u8; N]> for Rid<N> {
    fn from(arr: [u8; N]) -> Self {
        Self(arr)
    }
}
impl<const N: usize> PartialEq<[u8; N]> for Rid<N> {
    fn eq(&self, other: &[u8; N]) -> bool {
        &self.0 == other
    }
}
#[cfg(feature = "rkyv")]
mod rkyv_impls {
    use super::*;
    impl<const N: usize> ReplicaId for ArchivedRid<N> {}
    impl<const N: usize> ContentId for ArchivedRid<N> {
        fn from_hash(hash: Vec<u8>) -> Option<Self> {
            <[u8; N]>::try_from(hash).ok().map(Self)
        }
        fn len(&self) -> usize {
            self.0.len()
        }
    }
    impl<const N: usize> AsRef<[u8]> for ArchivedRid<N> {
        fn as_ref(&self) -> &[u8] {
            self.0.as_ref()
        }
    }
    impl<const N: usize> Clone for ArchivedRid<N> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<const N: usize> Debug for ArchivedRid<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // PERF: Can we fork multibase to make a non-allocating display? I would think
            // yes offhand, so i think this Display is okay for now - hoping that in the nearish
            // future we can provide an alt impl of encode that writes chars to the formatter
            // directly.
            write!(f, "Rid<{}>({})", self.0.len(), self.encode())
        }
    }
    impl<const N: usize> Display for ArchivedRid<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // PERF: Can we fork multibase to make a non-allocating display? I would think
            // yes offhand, so i think this Display is okay for now - hoping that in the nearish
            // future we can provide an alt impl of encode that writes chars to the formatter
            // directly.
            write!(f, "{}", self.encode())
        }
    }
    impl<const N: usize> PartialEq for ArchivedRid<N> {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq(&other.0)
        }
    }
    impl<const N: usize> Eq for ArchivedRid<N> {}
    impl<const N: usize> PartialOrd for ArchivedRid<N> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.0.partial_cmp(&other.0)
        }
    }
    impl<const N: usize> Ord for ArchivedRid<N> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }
    }
}
