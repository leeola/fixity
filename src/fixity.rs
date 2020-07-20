use {
    crate::{
        storage::Storage, store::StoreBorsh, Addr, Addr, BytesAddrs, BytesBlobs, BytesHeader,
        BytesNode, BytesPart, Error, Result, Store,
    },
    fastcdc::Chunk,
    multibase::Base,
    std::{
        io::{Read, Write},
        mem,
    },
};
pub const CDC_MIN: usize = 1024 * 16;
pub const CDC_AVG: usize = 1024 * 32;
pub const CDC_MAX: usize = 1024 * 64;
const MAX_ADDRS: usize = u8::MAX as usize;
pub struct Fixity<S> {
    storage: S,
    cdc_min: usize,
    cdc_avg: usize,
    cdc_max: usize,
    branch_width: usize,
}
impl<S> Fixity<S> {
    pub fn builder() -> Builder<S> {
        Builder::new()
    }
}
impl<S> Fixity<S>
where
    S: Storage,
{
    fn recursive_tree(
        &self,
        depth: usize,
        data: &[u8],
        iter: &mut impl Iterator<Item = Chunk>,
    ) -> Result<BytesPart> {
        if depth > 0 {
            let mut bytes_count = 0;
            let mut addrs = Vec::with_capacity(self.branch_width);
            for _ in 0..self.branch_width {
                let part = self.recursive_tree(depth - 1, data, iter)?;
                if part.addrs.is_empty() {
                    break;
                }
                bytes_count += part.bytes_count;
                let addr = self.put(&part)?;
                addrs.push(addr);
                if part.addrs.len() <= self.branch_width {
                    break;
                }
            }
            return Ok(BytesPart {
                bytes_count,
                addrs: BytesAddrs::Parts(addrs),
            });
        }

        let mut bytes_count = 0;
        let mut leafs = Vec::with_capacity(self.branch_width);
        for _ in 0..self.branch_width {
            let leaf = self.leaf(data, iter)?;
            if leaf.blobs.is_empty() {
                break;
            }
            bytes_count += leaf.bytes_count;
            let addr = self.put(&leaf)?;
            leafs.push(addr);
        }
        Ok(BytesPart {
            bytes_count: bytes_count as u64,
            addrs: BytesAddrs::Blobs(leafs),
        })
    }
    fn branch(&self, data: &[u8], iter: &mut impl Iterator<Item = Chunk>) -> Result<BytesPart> {
        let mut bytes_count = 0;
        let mut leafs = Vec::with_capacity(self.branch_width);
        for _ in 0..self.branch_width {
            let leaf = self.leaf(data, iter)?;
            if leaf.blobs.is_empty() {
                break;
            }
            bytes_count += leaf.bytes_count;
            let addr = self.put(&leaf)?;
            leafs.push(addr);
        }
        Ok(BytesPart {
            bytes_count: bytes_count as u64,
            addrs: BytesAddrs::Blobs(leafs),
        })
    }
    fn leaf(&self, data: &[u8], iter: &mut impl Iterator<Item = Chunk>) -> Result<BytesBlobs> {
        let mut bytes_count = 0;
        let mut blobs = Vec::with_capacity(self.branch_width);
        for _ in 0..self.branch_width {
            let Chunk { offset, length } = match iter.next() {
                Some(c) => c,
                None => break,
            };
            let chunk = &data[offset..offset + length];
            let addr = self.put_chunk(&chunk)?;
            bytes_count += length;
            blobs.push(addr);
        }
        Ok(BytesBlobs {
            bytes_count: bytes_count as u64,
            blobs,
        })
    }
    fn put_chunk(&self, chunk: &dyn AsRef<[u8]>) -> Result<Addr> {
        let chunk = chunk.as_ref();
        // TODO: integrate blake3 into multihash repo, but using blake3 for now
        // to test it. Ideally we want multihash prefix suppport.
        let hash = <[u8; 32]>::from(blake3::hash(&chunk));
        let addr = multibase::encode(Base::Base58Btc, &chunk);
        let size = self.storage.write(&addr, &mut chunk)?;
        if size != chunk.len() {
            return Err(Error::IncompleteWrite {
                got: size,
                expected: chunk.len(),
            });
        }
        Ok(addr.into())
    }
}
impl<S> Store for Fixity<S>
where
    S: Storage,
{
    fn put_read(&self, r: &dyn Read) -> Result<Addr> {
        // let mut b = Vec::new();
        // // I don't think len can ever differ from the Vec len..?
        // let _ = r
        //     .read_to_end(&mut b)
        //     .map_err(|err| Error::IoInputRead { err })?;
        // // TODO: use chunked streaming once this [1] is fixed/merged:
        // // [1]: https://github.com/nlfiedler/fastcdc-rs/issues/3
        // let chunker = fastcdc::FastCDC::new(&b, self.cdc_min, self.cdc_avg, self.cdc_max);
        // let mut first_part = None;
        // let mut bytes_count;
        // let mut blob_count;
        // let mut parts_bytes_count;
        // let mut part_bytes_count;
        // let mut layer = Vec::new();
        // let mut parts = Vec::new();
        // let mut blobs = Vec::new();
        // let mut layer = 1;
        // let mut blob_layer_limit = MAX_ADDRS.pow(layer);
        // for (i, Chunk { offset, length }) in chunker.enumerate() {
        //     blob_count += 1;
        //     part_bytes_count += length;
        //     let chunk = &b[offset..offset + length];
        //     let addr = self.put_chunk(&chunk)?;
        //     log::trace!(
        //         "chunk#{} addr:{:?}, offset:{}, size:{}",
        //         i,
        //         addr,
        //         offset,
        //         length
        //     );
        //     blobs.push(addr);
        //     if blobs.len() == MAX_ADDRS {
        //         parts_bytes_count += part_bytes_count;
        //         parts.push(BytesPart {
        //             bytes_count: part_bytes_count,
        //             blobs,
        //         })
        //     if parts.len() == MAX_ADDRS {

        //     }
        //     }
        // }
        todo!()
    }
}
// fn part(&self, b: &[u8], chunks: Iter) -> Result<BytesPart> {
//     chunks.take_n(MAX_ADDRS).map(|Chunk{offset, length}| {
//             let chunk = &b[offset..offset + length];
//             let addr = self.put_chunk(&chunk)?;
// (length, addr)
//     }).fold((0,Vec<Addr>), |(part_bytes_count, mut addrs), (blob_bytes_count, addr)| {
//         addrs.push(addr);
// (part_bytes_count+blob_bytes_count, addrs)
//     });
// }
// fn layer(&self, b: &[u8], chunks: Iter) -> Result<BytesLayerPart>
pub struct Builder<S> {
    storage: Option<S>,
}
impl<S> Builder<S> {
    pub fn new() -> Self {
        Self { storage: None }
    }
    pub fn with_storage(mut self, storage: S) -> Self {
        self.storage.replace(storage);
        self
    }
    pub fn build(self) -> Result<Fixity<S>> {
        let storage = self.storage.ok_or_else(|| Error::Builder {
            message: "must call Builder::with_storage to build".into(),
        })?;
        Ok(Fixity {
            storage,
            cdc_min: CDC_MIN,
            cdc_avg: CDC_AVG,
            cdc_max: CDC_MAX,
            branch_width: 2,
        })
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::storage::{Memory, StorageRead, StorageWrite},
    };
    #[test]
    fn put() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();

        let fixi = Fixity::builder()
            .with_storage(Memory::default())
            .build()
            .unwrap();
        let mut hashes = Vec::new();
        fixi.put(&mut "foobarbaz".as_bytes(), &mut hashes).unwrap();
        dbg!(String::from_utf8(hashes).unwrap());
    }
}
