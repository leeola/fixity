use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            primitive::{BytesCreate, BytesRead},
            workspace::{AsWorkspaceRef, Guard, Workspace},
        },
        Addr, Error, Path,
    },
    tokio::io::{AsyncRead, AsyncWrite},
};
pub struct Bytes<'f, C, W> {
    storage: &'f C,
    workspace: &'f W,
    path: Path,
}
impl<'f, C, W> Bytes<'f, C, W> {
    pub fn new(storage: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
    pub async fn read<Writer>(&self, w: Writer) -> Result<Option<u64>, Error>
    where
        C: CacheRead,
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
    pub async fn write<Reader>(&self, r: Reader) -> Result<Addr, Error>
    where
        C: CacheRead + CacheWrite,
        W: Workspace,
        Reader: AsyncRead + Unpin + Send,
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
impl<C, W> AsWorkspaceRef for Bytes<'_, C, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn as_workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<C, W> AsCacheRef for Bytes<'_, C, W>
where
    C: CacheRead + CacheWrite,
{
    type Cache = C;
    fn as_cache_ref(&self) -> &Self::Cache {
        &self.storage
    }
}
