use multibase::Base;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};
use thiserror::Error;

pub trait RandReplicaBuf {
    fn new(&mut self, len: usize) -> Vec<u8>;
}
// TODO: Add Container impls?
pub trait ReplicaId:
    Clone + Sized + Send + Sync + Eq + Ord + Hash + Debug + Display + 'static
{
    type Buf<'a>: AsRef<[u8]>;
    fn new<R: RandReplicaBuf>(rand: &mut R) -> Result<Self, FromBufError>;
    /// Construct a replica identifier from the given buffer.
    fn from_buf(buf: Vec<u8>) -> Result<Self, FromBufError>;
    fn as_buf(&self) -> Self::Buf<'_>;
    /// Encode this `ReplicaId` as a string.
    fn encode(&self) -> String;
    /// Construct a `ReplicaId` from an encoded string. The encoding is expected to be that of it's
    /// own construction, aka the returned value of [`Self::encode`].
    //
    // TODO: Error type? :thinking:
    fn decode(encoded: &str) -> Result<Self, FromBufError>;
    fn len(&self) -> usize {
        self.as_buf().as_ref().len()
    }
}
#[derive(Error, Debug)]
pub enum FromBufError {
    #[error("invalid length")]
    Length,
}

pub const DEFAULT_RID_LENGTH: usize = 32;
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// TODO: Serde doesn't impl for const :(. Can i impl manually perhaps?
// #[cfg(feature = "serde")]
// #[derive(serde::Deserialize, serde::Serialize)]
#[cfg(feature = "rkyv")]
#[derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
#[cfg(feature = "rkyv")]
#[archive(compare(PartialEq, PartialOrd))]
// FIXME: Derives should work.. right? But they weren't for some reason, so i impl'd manually.
// #[cfg(feature = "rkyv")]
// #[archive_attr(derive(Hash))]
// #[cfg(feature = "rkyv")]
// #[archive_attr(derive(From))]
// TODO: Remove length param.
pub struct Rid<const N: usize = DEFAULT_RID_LENGTH>([u8; N]);
impl<const N: usize> ReplicaId for Rid<N> {
    type Buf<'a> = &'a [u8; N];
    fn new<R: RandReplicaBuf>(rand: &mut R) -> Result<Self, FromBufError> {
        Self::from_buf(rand.new(N))
    }
    fn from_buf(buf: Vec<u8>) -> Result<Self, FromBufError> {
        let inner = <[u8; N]>::try_from(buf).map_err(|_| FromBufError::Length)?;
        Ok(Self(inner))
    }
    fn as_buf(&self) -> Self::Buf<'_> {
        &self.0
    }
    fn encode(&self) -> String {
        multibase::encode(Base::Base58Btc, self.as_buf())
    }
    fn decode(encoded: &str) -> Result<Self, FromBufError> {
        let (_, buf) = multibase::decode(encoded).unwrap();
        <Self as ReplicaId>::from_buf(buf)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl Default for Rid<DEFAULT_RID_LENGTH>
where
    [u8; DEFAULT_RID_LENGTH]: Default,
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
        write!(f, "Rid({})", self.encode())
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
    use std::hash::Hasher;

    use super::*;
    impl<const N: usize> AsRef<[u8]> for ArchivedRid<N> {
        fn as_ref(&self) -> &[u8] {
            self.0.as_ref()
        }
    }
    impl<const N: usize> Clone for ArchivedRid<N> {
        fn clone(&self) -> Self {
            Self(self.0)
        }
    }
    impl<const N: usize> ReplicaId for ArchivedRid<N> {
        type Buf<'a> = &'a [u8; N];
        fn new<R: RandReplicaBuf>(rand: &mut R) -> Result<Self, FromBufError> {
            Self::from_buf(rand.new(N))
        }
        fn from_buf(buf: Vec<u8>) -> Result<Self, FromBufError> {
            let inner = <[u8; N]>::try_from(buf).map_err(|_| FromBufError::Length)?;
            Ok(Self(inner))
        }
        fn as_buf(&self) -> Self::Buf<'_> {
            &self.0
        }
        fn encode(&self) -> String {
            multibase::encode(Base::Base58Btc, self.as_buf())
        }
        fn decode(encoded: &str) -> Result<Self, FromBufError> {
            let (_, buf) = multibase::decode(encoded).unwrap();
            <Self as ReplicaId>::from_buf(buf)
        }
        fn len(&self) -> usize {
            self.0.len()
        }
    }
    impl<const N: usize> Hash for ArchivedRid<N> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }
    impl<const N: usize> Debug for ArchivedRid<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // PERF: Can we fork multibase to make a non-allocating display? I would think
            // yes offhand, so i think this Display is okay for now - hoping that in the nearish
            // future we can provide an alt impl of encode that writes chars to the formatter
            // directly.
            write!(f, "Rid({})", self.encode())
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
#[cfg(any(test, feature = "test"))]
mod test_rids {
    //! Test focused `ReplicaId` implementations over integers and conversions from integers
    //! for `Rid<N>`.
    //!
    //! ## Endian
    //! Note that all integer representations use Big Endian to ensure stable representations
    //! and thus Replica IDs when written to test stores.
    use super::Rid;

    // TODO: macro these impls.
    impl<const N: usize> From<i32> for Rid<N> {
        fn from(i: i32) -> Self {
            let mut buf = [0; N];
            let size = N.min((i32::BITS / 8) as usize);
            buf[..size].copy_from_slice(&i.to_be_bytes()[..size]);
            Self(buf)
        }
    }
    impl<const N: usize> From<i64> for Rid<N> {
        fn from(i: i64) -> Self {
            let mut buf = [0; N];
            let size = N.min((i64::BITS / 8) as usize);
            buf[..size].copy_from_slice(&i.to_be_bytes()[..size]);
            Self(buf)
        }
    }
}
