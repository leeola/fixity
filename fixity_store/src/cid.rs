use {multihash::MultihashDigest, std::convert::TryFrom};

pub const CID_LENGTH: usize = 34;

pub trait ContentId: Clone + Sized + Send + Sync + Eq + Ord {
    fn len(&self) -> usize;
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
impl<Cid> ContentHasher<Cid> for Hasher {
    fn hash(&self, buf: &[u8]) -> Cid {
        let hash = multihash::Code::from(*self).digest(&buf).to_bytes();
        match hash.try_into() {
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
