pub mod multihash_256;

use crate::{
    deser::{Deserialize, Serialize},
    type_desc::TypeDescription,
};
use multibase::Base;
use multihash::MultihashDigest;
use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    hash::Hash,
};
use thiserror::Error;

pub const CID_LENGTH: usize = 34;

pub trait NewContentId:
    Clone + Sized + Send + Sync + Eq + Ord + Hash + Debug + Display + 'static + TypeDescription
{
    type Hash<'a>: AsRef<[u8]>;
    /// Hash the given bytes and producing a content identifier.
    fn hash(buf: &[u8]) -> Self;
    /// Construct a content identifier from the given hash.
    fn from_hash(hash: Vec<u8>) -> Result<Self, FromHashError>;
    fn as_hash(&self) -> Self::Hash<'_>;
    fn size(&self) -> usize {
        self.as_hash().as_ref().len()
    }
}
pub trait ContentIdDeser<Deser>: NewContentId + Serialize<Deser> + Deserialize<Deser> {}
impl<Deser, T> ContentIdDeser<Deser> for T where
    T: NewContentId + Serialize<Deser> + Deserialize<Deser>
{
}
#[derive(Error, Debug)]
pub enum FromHashError {
    #[error("invalid length")]
    Length,
}

pub trait ContentId:
    Clone + Sized + Send + Sync + Eq + Ord + AsRef<[u8]> + Debug + Display
{
    fn from_hash(hash: Vec<u8>) -> Option<Self>;
    fn len(&self) -> usize;
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
    /// Encode this `ContentId` as a string.
    fn encode(&self) -> Box<str> {
        // TODO: bind the encoder to generic param perhaps?
        // thereby letting the ContentStore or MetaStore choose the encoding.

        multibase::encode(Base::Base58Btc, self.as_bytes()).into_boxed_str()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// TODO: Serde doesn't impl for const :(. Can i impl manually perhaps?
// #[cfg(feature = "serde")]
// #[derive(serde::Deserialize, serde::Serialize)]
#[cfg(feature = "rkyv")]
#[derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct Cid<const N: usize>([u8; N]);
impl<const N: usize> ContentId for Cid<N> {
    fn from_hash(hash: Vec<u8>) -> Option<Self> {
        <[u8; N]>::try_from(hash).ok().map(Self)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<const N: usize> Debug for Cid<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // PERF: Can we fork multibase to make a non-allocating display? I would think
        // yes offhand, so i think this Display is okay for now - hoping that in the nearish
        // future we can provide an alt impl of encode that writes chars to the formatter
        // directly.
        write!(f, "Cid<{}>({})", self.0.len(), self.encode())
    }
}
impl<const N: usize> Display for Cid<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // PERF: Can we fork multibase to make a non-allocating display? I would think
        // yes offhand, so i think this Display is okay for now - hoping that in the nearish
        // future we can provide an alt impl of encode that writes chars to the formatter
        // directly.
        write!(f, "{}", self.encode())
    }
}
impl<const N: usize> AsRef<[u8]> for Cid<N> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl<const N: usize> From<[u8; N]> for Cid<N> {
    fn from(arr: [u8; N]) -> Self {
        Self(arr)
    }
}
impl<const N: usize> PartialEq<[u8; N]> for Cid<N> {
    fn eq(&self, other: &[u8; N]) -> bool {
        &self.0 == other
    }
}
pub trait ContainedCids<Cid: ContentId> {
    fn contained_cids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Cid> + Send + 'a>;
}
// NIT: Maybe move to a macro and explicitly impl for common types?
impl<T, C> ContainedCids<C> for T
where
    C: ContentId,
{
    fn contained_cids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a C> + Send + 'a> {
        Box::new(std::iter::empty())
    }
}
pub trait ContentHasher<Cid: ContentId>: Send + Sync {
    fn hash(&self, buf: &[u8]) -> Cid;
    // A future fn to describe the underlying hasher.
    // Length, algo, etc.
    // fn desc() -> HasherDesc;
}

/// A default impl of `ContentHasher` with various underlying
/// hashing algorithms.
#[derive(Debug, Copy, Clone)]
pub enum Hasher {
    Blake3_256,
}
impl<Cid> ContentHasher<Cid> for Hasher
where
    Cid: ContentId,
{
    fn hash(&self, buf: &[u8]) -> Cid {
        let hash = multihash::Code::from(*self).digest(&buf).to_bytes();
        match Cid::from_hash(hash) {
            Some(cid) => cid,
            None => {
                unreachable!("multihash header + 256 fits into 34bytes")
            },
        }
    }
}
impl Default for Hasher {
    fn default() -> Self {
        Self::Blake3_256
    }
}
impl From<Hasher> for multihash::Code {
    fn from(h: Hasher) -> Self {
        // NIT: using the Multihash derive might make this a bit more idiomatic,
        // just not sure offhand if there's a way to do that while ensuring
        // we use the same codes as multihash.
        match h {
            Hasher::Blake3_256 => multihash::Code::Blake3_256,
        }
    }
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use super::{FromHashError, NewContentId};
    use multihash::MultihashDigest;

    // TODO: macro these impls.

    impl NewContentId for i32 {
        type Hash<'a> = [u8; 4];
        fn hash(buf: &[u8]) -> Self {
            let mhash = multihash::Code::Blake2s128.digest(buf.as_ref());
            let digest = &mhash.digest()[0..4];
            Self::from_be_bytes(
                digest
                    .try_into()
                    .expect("Blake2s128 truncated to 4 bytes fits into a [u8; 4]"),
            )
        }
        fn from_hash(hash: Vec<u8>) -> Result<Self, super::FromHashError> {
            let hash = Self::Hash::try_from(hash).map_err(|_| FromHashError::Length)?;
            Ok(Self::from_be_bytes(hash))
        }
        fn as_hash(&self) -> Self::Hash<'static> {
            self.to_be_bytes()
        }
    }
    impl NewContentId for i64 {
        type Hash<'a> = [u8; 8];
        fn hash(buf: &[u8]) -> Self {
            let mhash = multihash::Code::Blake2s128.digest(buf.as_ref());
            let digest = &mhash.digest()[0..8];
            Self::from_be_bytes(
                digest
                    .try_into()
                    .expect("Blake2s128 truncated to 8 bytes fits into a [u8; 8]"),
            )
        }
        fn from_hash(hash: Vec<u8>) -> Result<Self, super::FromHashError> {
            let hash = Self::Hash::try_from(hash).map_err(|_| FromHashError::Length)?;
            Ok(Self::from_be_bytes(hash))
        }
        fn as_hash(&self) -> Self::Hash<'static> {
            self.to_be_bytes()
        }
    }
}
