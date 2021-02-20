use {
    crate::{
        path::Path,
        primitive::{BytesCreate, BytesRead},
        storage::{AsStorageRef, StorageRead, StorageWrite},
        workspace::{AsWorkspaceRef, Guard, Workspace},
        Addr, Error,
    },
    tokio::io::{AsyncRead, AsyncWrite},
};
/// Bytes reads and writes content defined chunks of data to the fixity store,
/// storing chunks within a tree structured list.
///
/// This data structure allows for ordered inserts in a content addressable system
/// with minimal read and write cost.
///
/// See also: [`prolly_list`](crate::primitive::prolly_list).
pub struct Bytes<'f, S, W> {
    storage: &'f S,
    workspace: &'f W,
    path: Path,
}
impl<'f, S, W> Bytes<'f, S, W> {
    pub fn new(storage: &'f S, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
    /// Read bytes from the underlying [`Path`].
    ///
    /// If the Path does not exist, `None` is returned, signifying no bytes read.
    /// Which can be meaningfully different than zero bytes read.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map,path::Path};
    /// let f = Fixity::memory();
    /// let mut b = f.bytes("foo");
    /// let mut buf = Vec::<u8>::new();
    /// let len = b.read(&mut buf).await.unwrap();
    /// assert_eq!(len, None);
    /// b.stage("bytes").await.unwrap();
    /// let len = b.read(&mut buf);
    /// assert_eq!(len, Some(5));
    /// assert_eq!(buf, "bytes".as_bytes());
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Internal`]
    pub async fn read<Writer>(&self, w: Writer) -> Result<Option<u64>, Error>
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
    /// Write the given bytes to storage, staging the result into the active `Workspace`.
    ///
    /// # Errors
    ///
    /// - [`Error::Internal`]
    pub async fn stage<R>(&self, r: R) -> Result<Addr, Error>
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
impl<S, W> AsWorkspaceRef for Bytes<'_, S, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn as_workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<S, W> AsStorageRef for Bytes<'_, S, W>
where
    S: StorageRead + StorageWrite,
{
    type Storage = S;
    fn as_storage_ref(&self) -> &Self::Storage {
        &self.storage
    }
}
