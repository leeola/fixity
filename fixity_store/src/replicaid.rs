use std::fmt::Display;

use crate::contentid::ContentId;

pub trait ReplicaId: ContentId {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
