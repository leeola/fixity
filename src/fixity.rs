use {
    crate::{
        path::IntoPath,
        primitive::CommitLog,
        storage::{self, fs::Config as FsConfig, AsStorageRef, Fs},
        workspace::{self, AsWorkspaceRef, Guard, Status, Workspace},
        Addr, Bytes, Error, Map, Path, Storage,
    },
    std::path::PathBuf,
    tokio::{
        fs::{self},
        io,
    },
};
const FIXI_DIR_NAME: &str = ".fixi";
/// The primary root interface to the fixity store; a thin wrapper around the storage `S` and the
/// workspace `W`.
pub struct Fixity<S, W> {
    storage: S,
    workspace: W,
}
impl<S, W> Fixity<S, W> {
    /// Construct a [`Builder`] for `Fixity`.
    #[must_use]
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
    /// Create a **testing focused** instance of Fixity, with in-memory storage
    /// and workspace.
    ///
    /// # IMPORTANT
    ///
    /// This instance does not save data. Both the storage and the workspace are
    /// in-memory only, and **will be lost** when this instance is dropped.
    #[must_use]
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
    /// Return a `Map` interface at the given `Path` within the fixity store.
    pub fn map(&self, path: Path) -> Map<'_, S, W> {
        Map::new(&self.storage, &self.workspace, path)
    }
    /// Return a `Bytes` interface at the given `Path` within the fixity store.
    ///
    /// # Errors
    ///
    /// - [`Error::CannotReplaceRootMap`]
    pub fn bytes<T>(&self, path: T) -> Result<Bytes<'_, S, W>, Error>
    where
        T: IntoPath,
    {
        let path = path.into_path();
        if path.is_empty() {
            return Err(Error::CannotReplaceRootMap);
        }
        if !path.is_root_map() {
            return Err(Error::CannotReplaceRootMap);
        }
        Ok(Bytes::new(&self.storage, &self.workspace, path))
    }
}
/// A builder pattern for the `Fixity` instance.
///
/// # Warning
///
/// This UX is awkward due to the generic types, so it may be nuked in the future.
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
            },
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
            },
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
            },
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
impl<S, W> AsWorkspaceRef for Fixity<S, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn as_workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<S, W> AsStorageRef for Fixity<S, W>
where
    S: Storage,
{
    type Storage = S;
    fn as_storage_ref(&self) -> &Self::Storage {
        &self.storage
    }
}
/// A trait to describe a `T` that can write a new commit log to storage, and
/// update the workspace pointer to the newly writen commit log.
///
/// This trait is usually implemented on root `Fixity` interfaces, such as
/// [`Map`] and [`Bytes`], along with [`Fixity`] itself.
#[async_trait::async_trait]
pub trait Commit {
    /// Commit any staged changes to storage, and update the workspace pointer
    /// to match.
    async fn commit(&self) -> Result<Addr, Error>;
}
#[async_trait::async_trait]
impl<T> Commit for T
where
    T: AsWorkspaceRef + AsStorageRef + Sync,
{
    async fn commit(&self) -> Result<Addr, Error> {
        let storage = self.as_storage_ref();
        let workspace = self.as_workspace_ref();
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
            },
        };
        let mut commit_log = CommitLog::new(storage, commit_addr);
        let commit_addr = commit_log.append(staged_content).await?;
        workspace_guard.commit(commit_addr.clone()).await?;
        Ok(commit_addr)
    }
}
