use multihash::MultihashDigest;
use std::convert::TryFrom;

use crate::Error;

pub const CID_LENGTH: usize = 34;

pub trait ContentId: Clone + Sized + Send + Sync + Eq + Ord + AsRef<[u8]> {
    fn from_hash(hash: Vec<u8>) -> Result<Self, Error>;
    fn len(&self) -> usize;
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
impl<const N: usize> ContentId for [u8; N] {
    fn from_hash(hash: Vec<u8>) -> Result<Self, Error> {
        Self::try_from(hash).map_err(|_| ())
    }
    fn len(&self) -> usize {
        0
        // Self::len(self)
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
            Ok(cid) => cid,
            Err(_) => {
                // NIT:
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
