use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite, DeserCache},
            primitive::CommitLog,
            storage,
            workspace::{self, AsWorkspaceRef, Guard, Status, Workspace},
            Bytes, Map,
        },
        Addr, Error, Path,
    },
    tokio::io,
};
pub struct Fixity<C, W> {
    storage: C,
    workspace: W,
}
impl<C, W> Fixity<C, W> {
    /// Create a fixity instance from the provided workspace and storage.
    ///
    /// If you're not intending to provide your own cache / workspace,
    /// see also [`crate::Fixity::builder`].
    pub fn new(storage: C, workspace: W) -> Self {
        Self { storage, workspace }
    }
    /// Take ownership of the underlying cache.
    ///
    /// Useful for interacting directly with underlying [Primitives](crate::core::primitive).
    pub fn into_cache(self) -> C {
        self.into_cw().0
    }
    /// Take ownership of the underlying cache/workspace.
    ///
    /// Useful for interacting directly with underlying [Primitives](crate::core::primitive).
    pub fn into_cw(self) -> (C, W) {
        (self.storage, self.workspace)
    }
}
impl Fixity<DeserCache<()>, workspace::Memory> {
    /// Create a **testing focused** instance of Fixity, with in-memory storage
    /// and workspace.
    ///
    /// # IMPORTANT
    ///
    /// This instance does not save data. Both the storage and the workspace are
    /// in-memory only, and **will be lost** when this instance is dropped.
    pub fn memory() -> Self {
        Self {
            storage: DeserCache::new(()),
            workspace: workspace::Memory::new("default".to_owned()),
        }
    }
}
impl<C, W> Fixity<C, W>
where
    C: CacheRead + CacheWrite,
{
    pub fn map(&self, path: Path) -> Map<'_, C, W> {
        Map::new(&self.storage, &self.workspace, path)
    }
    pub fn bytes(&self, path: Path) -> Result<Bytes<'_, C, W>, Error> {
        if path.is_empty() {
            return Err(Error::CannotReplaceRootMap);
        }
        if !path.is_root_map() {
            return Err(Error::CannotReplaceRootMap);
        }
        Ok(Bytes::new(&self.storage, &self.workspace, path))
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
impl<C, W> AsWorkspaceRef for Fixity<C, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn as_workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<C, W> AsCacheRef for Fixity<C, W>
where
    C: CacheRead + CacheWrite,
{
    type Cache = C;
    fn as_cache_ref(&self) -> &Self::Cache {
        &self.storage
    }
}
/// A trait to describe a `T` that can write a new commit log to storage, and
/// update the workspace pointer to the newly writen commit log.
///
/// This trait is usually implemented on core `Fixity` interfaces, such as
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
    T: AsWorkspaceRef + AsCacheRef + Sync,
{
    async fn commit(&self) -> Result<Addr, Error> {
        let storage = self.as_cache_ref();
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
