use {
    crate::{
        error::TypeError,
        path::{Path, SegmentResolve, SegmentUpdate},
        primitive::{commitlog::CommitLog, prolly::refimpl, BytesCreate, BytesRead},
        storage::{StorageRead, StorageRef, StorageWrite},
        value::{Key, Value},
        workspace::{Guard, Status, Workspace, WorkspaceRef},
        Addr, Error,
    },
    std::{fmt, mem},
    tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite},
};
pub struct Bytes<'f, S, W> {
    storage: &'f S,
    workspace: &'f W,
    path: Path,
    staged_content: Option<Addr>,
}
impl<'f, S, W> Bytes<'f, S, W> {
    pub fn new(storage: &'f S, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
            staged_content: None,
        }
    }
    pub async fn read<Writer>(&self, mut w: Writer) -> Result<Option<u64>, Error>
    where
        S: StorageRead,
        W: Workspace,
        Writer: AsyncWrite + Unpin + Send,
    {
        let workspace_guard = self.workspace.lock().await?;
        let root_content_addr = workspace_guard
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let content_addr = self
            .path
            .resolve_last(self.storage, root_content_addr)
            .await?;
        let reader = if let Some(content_addr) = content_addr {
            BytesRead::new(self.storage, content_addr)
        } else {
            return Ok(None);
        };
        let n = reader.read(w).await?;
        Ok(Some(n))
    }
    pub async fn stage<R>(&self, mut r: R) -> Result<Addr, Error>
    where
        S: StorageRead + StorageWrite,
        W: Workspace,
        R: AsyncRead + Unpin + Send,
    {
        let workspace_guard = self.workspace.lock().await?;
        let root_content_addr = workspace_guard
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let resolved_path = self.path.resolve(self.storage, root_content_addr).await?;
        let new_self_addr = BytesCreate::new(self.storage).write(r).await?;
        let new_staged_content = self
            .path
            .update(self.storage, resolved_path, new_self_addr)
            .await?;
        workspace_guard.stage(new_staged_content.clone()).await?;
        Ok(new_staged_content)
    }
}
impl<S, W> WorkspaceRef for Bytes<'_, S, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<S, W> StorageRef for Bytes<'_, S, W>
where
    S: StorageRead + StorageWrite,
{
    type Storage = S;
    fn storage_ref(&self) -> &Self::Storage {
        &self.storage
    }
}