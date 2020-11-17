pub mod map;
pub use map::Map;

use {
    crate::{
        head::{Guard, Head},
        storage::{self, fs::Config as FsConfig, Fs, Storage},
        value::{Addr, Key, Path},
        Error, StorageWrite,
    },
    multibase::Base,
    std::path::PathBuf,
    tokio::{
        fs::{self},
        io::{self, AsyncRead},
    },
};

pub struct Fixity<S> {
    storage: S,
    fixity_dir: PathBuf,
    workspace: String,
}
impl Fixity<Fs> {
    /// Initialize a new Fixity repository.
    pub async fn init(
        fixity_dir: PathBuf,
        workspace: String,
        fs_config: FsConfig,
    ) -> Result<Self, Error> {
        fs::create_dir(&fixity_dir)
            .await
            .map_err(|source| InitError::CreateDir { source })?;
        let storage = Fs::init(fs_config)
            .await
            .map_err(|source| InitError::Storage { source })?;
        Ok(Self {
            fixity_dir,
            workspace,
            storage,
        })
    }
    /// Open an existing Fixity repository.
    pub async fn open(
        fixity_dir: PathBuf,
        workspace: String,
        fs_config: FsConfig,
    ) -> Result<Self, Error> {
        Ok(Self {
            fixity_dir,
            workspace,
            storage: Fs::open(fs_config)?,
        })
    }
}
impl<S> Fixity<S> {
    pub fn new() -> Builder<S> {
        Builder::default()
    }
}
impl<S> Fixity<S>
where
    S: StorageWrite,
{
    pub async fn map<'f>(&'f self, path: Path) -> Result<Guard<Map<'f, S>>, Error> {
        // TODO: recursively load Map's until the Path is met.
        if !path.is_empty() {
            unimplemented!("opening a map with a path");
        }

        let head = Head::open(self.fixity_dir.as_path(), self.workspace.as_str()).await?;
        let inner = Map::new(&self.storage, head.addr());
        Ok(Guard::new(head, inner))
    }
    pub async fn put_reader<R>(&self, mut r: R) -> Result<String, Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        log::warn!("putting without chunking");
        let mut bytes = Vec::new();
        io::copy(&mut r, &mut bytes).await?;
        let hash = <[u8; 32]>::from(blake3::hash(&bytes));
        let addr = multibase::encode(Base::Base58Btc, &hash);
        let n = self.storage.write(addr.clone(), r).await?;
        log::trace!("{} bytes written to {}", n, addr);
        Ok(addr)
    }
}
pub struct Builder<S> {
    fixity_dir: Option<PathBuf>,
    storage: Option<S>,
    workspace: Option<String>,
}
impl<S> Default for Builder<S> {
    fn default() -> Self {
        Self {
            storage: None,
            fixity_dir: None,
            workspace: None,
        }
    }
}
impl<S> Builder<S> {
    pub fn with_storage(mut self, storage: S) -> Self {
        self.storage.replace(storage);
        self
    }
    pub fn with_fixity_dir(mut self, fixity_dir: PathBuf) -> Self {
        self.fixity_dir.replace(fixity_dir);
        self
    }
    pub fn with_workspace(mut self, workspace: String) -> Self {
        self.workspace.replace(workspace);
        self
    }
    pub fn build(self) -> Result<Fixity<S>, Error> {
        let fixity_dir = self.fixity_dir.ok_or_else(|| Error::Builder {
            message: "missing fixity_dir".into(),
        })?;
        let storage = self.storage.ok_or_else(|| Error::Builder {
            message: "missing storage".into(),
        })?;
        let workspace = self.workspace.ok_or_else(|| Error::Builder {
            message: "missing workspace".into(),
        })?;
        Ok(Fixity {
            storage,
            fixity_dir,
            workspace,
        })
    }
}
#[async_trait::async_trait]
pub trait Flush {
    async fn flush(&mut self) -> Result<Addr, Error>;
}
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("failed creating fixity directory: `{source}`")]
    CreateDir { source: io::Error },
    #[error("failed creating new storage: `{source}`")]
    Storage {
        #[from]
        source: storage::Error,
    },
}
/*
pub mod table;
pub use table::Table;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        hash_tree, storage::Storage, Addr, ContentAddrs, ContentHeader, ContentNode, Error, Result,
        Store,
    },
    fastcdc::Chunk,
    multibase::Base,
    std::{
        io::{Read, Write},
        mem,
    },
};
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Scalar {
    Uint32(u32),
    Ref(Ref),
}
impl From<u32> for Scalar {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
    }
}
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Value {
    Uint32(u32),
    Ref(Ref),
    // Map(Map),
}
impl<T> From<T> for Value
where
    T: Into<Scalar>,
{
    fn from(t: T) -> Self {
        match t.into() {
            Scalar::Uint32(v) => Self::Uint32(v),
            Scalar::Ref(v) => Self::Ref(v),
        }
    }
}
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Ref {
    Map(Addr),
}
pub const CDC_MIN: usize = 1024 * 16;
pub const CDC_AVG: usize = 1024 * 32;
pub const CDC_MAX: usize = 1024 * 64;
const MAX_ADDRS: usize = u8::MAX as usize;
pub enum Entry<'s, S, T> {
    Vacant(VacantEntry<'s, S, T>),
    Occupied(OccupiedEntry<T>),
}
impl<'s, S, T> Entry<'s, S, T>
where
    T: From<VacantEntry<'s, S, T>>,
{
    pub fn inner(self) -> T {
        match self {
            Self::Occupied(o) => o.inner(),
            Self::Vacant(v) => v.inner(),
        }
    }
}
pub struct VacantEntry<'s, S, T> {
    storage: &'s S,
    _phantom: std::marker::PhantomData<T>,
}
impl<'s, S, T> VacantEntry<'s, S, T>
where
    T: From<Self>,
{
    pub fn inner(self) -> T {
        T::from(self)
    }
}
impl<'s, S> From<VacantEntry<'s, S, Map<'s, S>>> for Map<'s, S> {
    fn from(ve: VacantEntry<'s, S, Map<'s, S>>) -> Self {
        Self::new(ve.storage)
    }
}
pub struct OccupiedEntry<T> {
    entry: T,
}
impl<T> OccupiedEntry<T> {
    pub fn inner(self) -> T {
        self.entry
    }
}
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
    fn map<'s, K>(&'s self, _k: K) -> Result<Entry<'s, S, Map<'s, S>>>
    where
        K: AsRef<str>,
    {
        todo!()
    }
    fn stage<K, V>(&self, _k: K, _v: V) -> Result<Addr>
    where
        K: AsRef<str>,
        V: Into<Value>,
    {
        todo!()
    }

    /*
    fn store_(
        &self,
        depth: usize,
        init_child: Option<BytesPart>,
        data: &[u8],
        iter: &mut impl Iterator<Item = Chunk>,
    ) -> Result<BytesPart> {
        if depth > 0 {
            let mut bytes_count = 0;
            let mut addrs = Vec::with_capacity(self.branch_width);
            if let Some(init_child) = init_child {
                bytes_count += init_child.bytes_count;
                let addr = self.put(&init_child)?;
                addrs.push(addr);
            }
            for _ in addrs.len()..self.branch_width {
                let part = self.recursive_tree(depth - 1, None, data, iter)?;
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
    fn foo(&self, data: &[u8], iter: &mut impl Iterator<Item = Chunk>) -> Result<BytesPart> {
        let depth = 0;
        let child = None;
        loop {
            let node = self.recursive_tree(depth, child, data, iter)?;
            if node.addrs.len() == self.branch_width {
                child = Some(node);
                depth += 1;
            } else {
                break Ok(node);
            }
        }
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
    */
    fn put_chunk(&self, chunk: &dyn AsRef<[u8]>) -> Result<Addr> {
        let mut chunk = chunk.as_ref();
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
    /*
    fn put_read(&self, r: &mut dyn Read) -> Result<Addr> {
        let mut b = Vec::new();
        // I don't think len can ever differ from the Vec len..?
        let _ = r
            .read_to_end(&mut b)
            .map_err(|err| Error::IoInputRead { err })?;
        // TODO: use chunked streaming once this [1] is fixed/merged:
        // [1]: https://github.com/nlfiedler/fastcdc-rs/issues/3
        let chunker = fastcdc::FastCDC::new(&b, self.cdc_min, self.cdc_avg, self.cdc_max);

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
    */
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
        // let hash = fixi.put_read(&mut "foobarbaz".as_bytes()).unwrap();
        // dbg!(hash);
    }
}
*/
