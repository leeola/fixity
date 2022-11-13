use super::{FromHashError, NewContentId};
use multibase::Base;
use multihash::MultihashDigest;
#[cfg(feature = "serde")]
use serde_big_array::BigArray;
use std::fmt::{Debug, Display};

const MULTIHASH_256_LEN: usize = 34;
const MULTIBASE_ENCODE: Base = Base::Base58Btc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg(feature = "serde")]
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg(feature = "rkyv")]
#[derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct Multihash256(
    #[cfg_attr(feature = "rkyv", serde(with = "BigArray"))] [u8; MULTIHASH_256_LEN],
);
impl NewContentId for Multihash256 {
    type Hash = [u8; MULTIHASH_256_LEN];
    fn hash<B: AsRef<[u8]>>(buf: B) -> Self {
        let hash = multihash::Code::Blake3_256.digest(buf.as_ref()).to_bytes();
        match Self::from_hash(hash) {
            Ok(cid) => cid,
            Err(_) => {
                unreachable!("Blake3_256 fits into 34bytes")
            },
        }
    }
    fn from_hash<H: TryInto<Self::Hash>>(hash: H) -> Result<Self, FromHashError> {
        hash.try_into()
            .map_or(Err(FromHashError::Length), |hash| Ok(Self(hash)))
    }
    fn as_hash(&self) -> &Self::Hash {
        &self.0
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl Debug for Multihash256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // PERF: Can we fork multibase to make a non-allocating display? I would think
        // yes offhand, so i think this Display is okay for now - hoping that in the nearish
        // future we can provide an alt impl of encode that writes chars to the formatter
        // directly.
        write!(
            f,
            "Multihash256({})",
            multibase::encode(MULTIBASE_ENCODE, &self.0)
        )
    }
}
impl Display for Multihash256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // PERF: Can we fork multibase to make a non-allocating display? I would think
        // yes offhand, so i think this Display is okay for now - hoping that in the nearish
        // future we can provide an alt impl of encode that writes chars to the formatter
        // directly.
        write!(f, "{}", multibase::encode(MULTIBASE_ENCODE, &self.0))
    }
}
impl AsRef<[u8]> for Multihash256 {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl From<[u8; MULTIHASH_256_LEN]> for Multihash256 {
    fn from(arr: [u8; MULTIHASH_256_LEN]) -> Self {
        Self(arr)
    }
}
impl PartialEq<[u8; MULTIHASH_256_LEN]> for Multihash256 {
    fn eq(&self, other: &[u8; MULTIHASH_256_LEN]) -> bool {
        &self.0 == other
    }
}
