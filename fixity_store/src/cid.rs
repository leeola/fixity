use multihash::MultihashDigest;

const CID_LENGTH: usize = 34;
pub type Cid = [u8; CID_LENGTH];

pub trait ContentHasher {
    type Cid;
    fn hash(&self, buf: &[u8]) -> Self::Cid;
    // A future fn to describe the underlying hasher.
    // Length, algo, etc.
    // fn desc() -> HasherDesc;
}

#[derive(Debug, Copy, Clone)]
pub enum Hashers {
    Blake3_256,
}
impl ContentHasher for Hashers {
    type Cid = Cid;
    fn hash(&self, buf: &[u8]) -> Self::Cid {
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
impl Default for Hashers {
    fn default() -> Self {
        Self::Blake3_256
    }
}
impl From<Hashers> for multihash::Code {
    fn from(h: Hashers) -> Self {
        // NIT: using the Multihash derive might make this a bit more idiomatic,
        // just not sure offhand if there's a way to do that while ensuring
        // we use the same codes as multihash.
        match h {
            Hashers::Blake3_256 => multihash::Code::Blake3_256,
        }
    }
}
