use {
    crate::{storage::Storage, Addr, Error, Result, Store},
    fastcdc::Chunk,
    multibase::Base,
    std::io::{Read, Write},
};
pub const CDC_MIN: usize = 1024 * 16;
pub const CDC_AVG: usize = 1024 * 32;
pub const CDC_MAX: usize = 1024 * 64;
pub struct Fixity<S> {
    storage: S,
    cdc_min: usize,
    cdc_avg: usize,
    cdc_max: usize,
}
impl<S> Fixity<S> {
    pub fn builder() -> Builder<S> {
        Builder::new()
    }
}
impl<S> Store for Fixity<S>
where
    S: Storage,
{
    fn put_chunk(&self, chunk: &dyn AsRef<[u8]>) -> Result<Addr> {
        let chunk = chunk.as_ref();
        // TODO: integrate blake3 into multihash repo, but using blake3 for now
        // to test it. Ideally we want multihash prefix suppport.
        let hash = <[u8; 32]>::from(blake3::hash(&chunk));
        let addr = multibase::encode(Base::Base58Btc, &chunk);
        let size = self.storage.write(&addr, &chunk)?;
        if size != chunk.len() {
            return Err(Error::IncompleteWrite {
                got: size,
                expected: chunk.len(),
            });
        }
        todo!()
    }
    fn put(&self, r: &mut dyn Read) -> Result<Addr> {
        let mut b = Vec::new();
        // I don't think len can ever differ from the Vec len..?
        let _ = r
            .read_to_end(&mut b)
            .map_err(|err| Error::IoInputRead { err })?;
        // TODO: use chunked streaming once this [1] is fixed/merged:
        // [1]: https://github.com/nlfiedler/fastcdc-rs/issues/3
        let chunker = fastcdc::FastCDC::new(&b, self.cdc_min, self.cdc_avg, self.cdc_max);
        for Chunk { offset, length } in chunker {
            let chunk = &b[offset..offset + length];
            // TODO: integrate blake3 into multihash repo, but using blake3 for now
            // to test it. Ideally we want multihash prefix suppport.
            let hash = <[u8; 32]>::from(blake3::hash(chunk));
            let addr = multibase::encode(Base::Base58Btc, chunk);
            log::trace!("chunk addr:{}, offset:{}, size:{}", addr, offset, length);
            addrs_w.write_all(addr.as_bytes())?;
        }
        Ok(len)
    }
}
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
