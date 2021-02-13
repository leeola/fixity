use {
    crate::{
        primitive::CommitLog,
        storage::{self, fs::Config as FsConfig, Fs, StorageRef},
        workspace::{self, Guard, Status, Workspace, WorkspaceRef},
        Addr, Bytes, Error, Map, Path, Storage,
    },
    multibase::Base,
    std::path::PathBuf,
    tokio::{
        fs::{self},
        io::{self, AsyncRead},
    },
};
const FIXI_DIR_NAME: &str = ".fixi";
pub struct Fixity<S, W> {
    storage: S,
    workspace: W,
}
impl<S, W> Fixity<S, W> {
    pub fn builder() -> Builder<S, W> {
        Builder::default()
    }
    /// Create a fixity instance from the provided workspace and storage.
    ///
    /// Most users will want to use [`Fixity::builder`].
    pub fn new(storage: S, workspace: W) -> Self {
        Self { storage, workspace }
    }
}
impl Fixity<storage::Memory, workspace::Memory> {
    /// Create a **testing focused** instance of Fixity, with in-memory storage and workspace.
    ///
    /// # IMPORTANT
    ///
    /// This instance does not save data. Both the storage and the workspace are in-memory
    /// only, and **will be lost** when this instance is dropped.
    pub fn memory() -> Fixity<storage::Memory, workspace::Memory> {
        Self {
            storage: storage::Memory::new(),
            workspace: workspace::Memory::new("default".to_owned()),
        }
    }
}
impl<S, W> Fixity<S, W>
where
    S: Storage,
{
    pub fn map(&self, path: Path) -> Map<'_, S, W> {
        Map::new(&self.storage, &self.workspace, path)
    }
    pub fn bytes(&self, path: Path) -> Result<Bytes<'_, S, W>, Error> {
        if path.is_empty() {
            return Err(Error::CannotReplaceRootMap);
        }
        if !path.is_root_map() {
            return Err(Error::CannotReplaceRootMap);
        }
        Ok(Bytes::new(&self.storage, &self.workspace, path))
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
pub struct Builder<S, W> {
    storage: Option<S>,
    workspace: Option<W>,
    fixi_dir_name: Option<PathBuf>,
    fixi_dir: Option<PathBuf>,
    fs_storage_dir: Option<PathBuf>,
    workspace_name: Option<String>,
}
impl<S, W> Default for Builder<S, W> {
    fn default() -> Self {
        Self {
            storage: None,
            workspace: None,
            fixi_dir_name: None,
            fixi_dir: None,
            fs_storage_dir: None,
            workspace_name: None,
        }
    }
}
impl<S, W> Builder<S, W> {
    pub fn with_storage(mut self, storage: S) -> Self {
        self.storage.replace(storage);
        self
    }
    pub fn with_workspace(mut self, workspace: W) -> Self {
        self.workspace.replace(workspace);
        self
    }
    pub fn fixi_dir_name(mut self, fixi_dir_name: Option<PathBuf>) -> Self {
        self.fixi_dir_name = fixi_dir_name;
        self
    }
    pub fn fixi_dir(mut self, fixi_dir: Option<PathBuf>) -> Self {
        self.fixi_dir = fixi_dir;
        self
    }
    pub fn workspace_name(mut self, workspace_name: Option<String>) -> Self {
        self.workspace_name = workspace_name;
        self
    }
    pub fn with_fixi_dir_name(mut self, fixi_dir_name: PathBuf) -> Self {
        self.fixi_dir_name.replace(fixi_dir_name);
        self
    }
    pub fn with_workspace_name(mut self, workspace_name: String) -> Self {
        self.workspace_name.replace(workspace_name);
        self
    }
    pub fn fs_storage_dir(mut self, fs_storage_dir: Option<PathBuf>) -> Self {
        self.fs_storage_dir = fs_storage_dir;
        self
    }
}
impl Builder<storage::Fs, workspace::Fs> {
    /// Initialize a new Fixity repository.
    pub async fn init(self) -> Result<Fixity<Fs, workspace::Fs>, Error> {
        let fixi_dir = match (self.fixi_dir_name, self.fixi_dir) {
            (_, Some(fixi_dir)) => fixi_dir,
            (fixi_dir_name, None) => fixi_dir_name.unwrap_or_else(|| PathBuf::from(FIXI_DIR_NAME)),
        };
        fs::create_dir(&fixi_dir)
            .await
            .map_err(|source| InitError::CreateDir { source })?;
        let storage = match (self.storage, self.fs_storage_dir) {
            (Some(storage), _) => storage,
            (None, fs_storage_dir) => Fs::init(FsConfig {
                path: fs_storage_dir.unwrap_or_else(|| fixi_dir.join("storage")),
            })
            .await
            .map_err(|source| InitError::Storage { source })?,
        };
        // init the Workspace
        let workspace = match self.workspace {
            Some(w) => w,
            None => {
                let workspace_name = self.workspace_name.unwrap_or_else(|| "default".to_owned());
                workspace::Fs::init(fixi_dir.join("workspaces"), workspace_name).await?
            }
        };
        Ok(Fixity::new(storage, workspace))
    }
    pub async fn open(self) -> Result<Fixity<Fs, workspace::Fs>, Error> {
        let fixi_dir = match (self.fixi_dir_name, self.fixi_dir) {
            (_, Some(fixi_dir)) => fixi_dir,
            (fixi_dir_name, None) => {
                let fixi_dir_name = fixi_dir_name.unwrap_or_else(|| PathBuf::from(FIXI_DIR_NAME));
                crate::dir::resolve(fixi_dir_name, PathBuf::from("."))
                    .ok_or(Error::RepositoryNotFound)?
            }
        };
        let storage = match (self.storage, self.fs_storage_dir) {
            (Some(storage), _) => storage,
            (None, fs_storage_dir) => Fs::open(FsConfig {
                path: fs_storage_dir.unwrap_or_else(|| fixi_dir.join("storage")),
            })?,
        };
        // open the Workspace
        let workspace = match self.workspace {
            Some(w) => w,
            None => {
                let workspace_name = self.workspace_name.unwrap_or_else(|| "default".to_owned());
                workspace::Fs::open(fixi_dir.join("workspaces"), workspace_name).await?
            }
        };
        Ok(Fixity::new(storage, workspace))
    }
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
impl<S, W> WorkspaceRef for Fixity<S, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<S, W> StorageRef for Fixity<S, W>
where
    S: Storage,
{
    type Storage = S;
    fn storage_ref(&self) -> &Self::Storage {
        &self.storage
    }
}
/// A trait to describe a `T` that can write a new commit log to storage, and update
/// the workspace pointer to the newly writen commit log.
///
/// This trait is usually implemented on root `Fixity` interfaces, such as [`Map`] and
/// [`Bytes`], along with [`Fixity`] itself.
#[async_trait::async_trait]
pub trait Commit {
    /// Commit any staged changes to storage, and update the workspace pointer to match.
    async fn commit(&self) -> Result<Addr, Error>;
}
#[async_trait::async_trait]
impl<T> Commit for T
where
    T: WorkspaceRef + StorageRef + Sync,
{
    async fn commit(&self) -> Result<Addr, Error> {
        let storage = self.storage_ref();
        let workspace = self.workspace_ref();
        let workspace_guard = workspace.lock().await?;
        let (commit_addr, staged_content) = match workspace_guard.status().await? {
            Status::InitStaged { staged_content, .. } => (None, staged_content),
            Status::Staged {
                commit,
                staged_content,
                ..
            } => (Some(commit), staged_content),
            Status::Detached(_) => return Err(Error::DetachedHead),
            Status::Init { .. } | Status::Clean { .. } => {
                return Err(Error::NoStageToCommit);
            }
        };
        let mut commit_log = CommitLog::new(storage, commit_addr);
        let commit_addr = commit_log.append(staged_content).await?;
        workspace_guard.commit(commit_addr.clone()).await?;
        Ok(commit_addr)
    }
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
