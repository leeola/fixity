pub mod multihash_256;

use crate::deser::{Deserialize, Serialize};
use multibase::Base;
use multihash::MultihashDigest;
use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    hash::Hash,
};
use thiserror::Error;

pub const CID_LENGTH: usize = 36;

pub trait NewContentId:
    Clone + Sized + Send + Sync + Eq + Ord + Hash + Debug + Display + 'static
{
    type Hash<'a>: AsRef<[u8]>;
    /// Hash the given bytes and producing a content identifier.
    fn hash(buf: &[u8]) -> Self;
    /// Construct a content identifier from the given hash.
    fn from_hash(hash: Vec<u8>) -> Result<Self, FromHashError>;
    /// Encode this `ContentId` as a string.
    fn encode(&self) -> String;
    /// Construct a `ContentId` from an encoded string. The encoding is expected to be that of it's
    /// own construction, aka the returned value of [`Self::encode`].
    //
    // TODO: Error type? :thinking:
    fn decode(encoded: &str) -> Result<Self, FromHashError>;
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
    fn encode(&self) -> String {
        multibase::encode(Base::Base58Btc, self.as_bytes())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// TODO: Serde doesn't impl for const :(. Can i impl manually perhaps?
// #[cfg(feature = "serde")]
// #[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)
)]
// TODO: Remove length param.
pub struct Cid<const N: usize = CID_LENGTH>([u8; N]);
impl<const N: usize> ContentId for Cid<N> {
    fn from_hash(hash: Vec<u8>) -> Option<Self> {
        <[u8; N]>::try_from(hash).ok().map(Self)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl NewContentId for Cid<CID_LENGTH> {
    type Hash<'a> = &'a [u8; CID_LENGTH];
    fn hash(buf: &[u8]) -> Self {
        let multihash = multihash::Code::Blake2b256.digest(buf);
        Self(
            multihash
                .to_bytes()
                .try_into()
                .expect("Blake2b256 fits into 36 bytes"),
        )
    }
    fn from_hash(hash: Vec<u8>) -> Result<Self, FromHashError> {
        let arr = <[u8; CID_LENGTH]>::try_from(hash).map_err(|_| FromHashError::Length)?;
        Ok(Self(arr))
    }
    fn encode(&self) -> String {
        multibase::encode(Base::Base58Btc, self.as_hash())
    }
    fn decode(encoded: &str) -> Result<Self, FromHashError> {
        let (_, buf) = multibase::decode(encoded).unwrap();
        <Self as NewContentId>::from_hash(buf)
    }
    fn as_hash(&self) -> Self::Hash<'_> {
        &self.0
    }
    fn size(&self) -> usize {
        self.0.len()
    }
}
impl<const N: usize> Default for Cid<N> {
    fn default() -> Self {
        Self([0; N])
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

#[cfg(any(test, feature = "test"))]
pub mod test {
    //! Test focused `ContentId` implementations over integers and conversions from integers
    //! for `Cid<N>`.
    //!
    //! ## Endian
    //! Note that all integer representations use Big Endian to ensure stable representations
    //! and thus Content IDs when written to test stores.
    use super::Cid;

    // TODO: macro these impls.
    impl<const N: usize> From<i32> for Cid<N> {
        fn from(i: i32) -> Self {
            let mut buf = [0; N];
            let size = N.min((i32::BITS / 8) as usize);
            buf[..size].copy_from_slice(&i.to_be_bytes()[..size]);
            Self(buf)
        }
    }
    impl<const N: usize> From<i64> for Cid<N> {
        fn from(i: i64) -> Self {
            let mut buf = [0; N];
            let size = N.min((i64::BITS / 8) as usize);
            buf[..size].copy_from_slice(&i.to_be_bytes()[..size]);
            Self(buf)
        }
    }
}
