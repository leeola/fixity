use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            misc::range_ext::{OwnedRangeBounds, RangeBoundsExt},
            primitive::{
                bytes::Create as BytesCreate,
                commitlog::CommitLog,
                map::refimpl::{Change as MapChange, Create as MapCreate, Update as MapUpdate},
            },
            workspace::{AsWorkspaceRef, Guard, Status, Workspace},
        },
        Addr, Error, Key, Path, Value,
    },
    std::{fmt, mem, ops::Bound},
    tokio::io::{AsyncRead, AsyncWrite},
};
pub struct GitLfs<'f, C, W> {
    cache: &'f C,
    workspace: &'f W,
    path: Path,
}
impl<'f, C, W> GitLfs<'f, C, W> {
    pub fn new(cache: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            cache,
            workspace,
            path,
        }
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
            .content_addr(self.cache)
            .await?;
        let resolved_path = self.path.resolve(self.cache, root_content_addr).await?;
        let old_map_addr = resolved_path
            .last()
            .cloned()
            .expect("resolved Path has zero len");
        let bytes_addr = BytesCreate::new(self.cache).write(r).await?;
        let bytes_len: u64 = todo!("bytes_len");
        // A separate checksum formatted as git-lfs expects it normally.
        // this is usually `"sha256:hash"`
        let checksum_key: String = todo!("checksum");
        let new_map_addr = if let Some(map_addr) = old_map_addr {
            let kvs = vec![(
                Key(Value::String(checksum_key)),
                MapChange::Insert(Value::Addr(bytes_addr)),
            )];
            MapUpdate::new(self.cache, map_addr).with_vec(kvs).await?
        } else {
            let kvs = vec![(Key(Value::String(checksum_key)), Value::Addr(bytes_addr))];
            MapCreate::new(self.cache).with_vec(kvs).await?
        };
        let new_staged_content = self
            .path
            .update(self.cache, resolved_path, new_map_addr)
            .await?;
        workspace_guard.stage(new_staged_content.clone()).await?;
        Ok(new_staged_content)
    }
}
