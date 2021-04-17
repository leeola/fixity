use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            primitive::{commitlog::CommitLog, prollytree::refimpl, BytesCreate, BytesRead},
            workspace::{AsWorkspaceRef, Guard, Workspace},
        },
        Addr, Error, Key, Path, Value,
    },
    tokio::io::{AsyncRead, AsyncWrite},
};
pub struct File<'f, C, W> {
    cache: &'f C,
    workspace: &'f W,
    path: Path,
}
impl<'f, C, W> File<'f, C, W> {
    pub fn new(cache: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            cache,
            workspace,
            path,
        }
    }
    pub async fn read_with_meta<Writer>(
        &self,
        writer: Writer,
    ) -> Result<Option<(u64, Metadata)>, Error>
    where
        C: CacheRead,
        W: Workspace,
        W: AsyncWrite + Unpin + Send,
    {
        todo!()
    }
    pub async fn write_with_meta<Reader>(
        &self,
        reader: Reader,
        metadata: Metadata,
    ) -> Result<Addr, Error>
    where
        C: CacheRead + CacheWrite,
        W: Workspace,
        Reader: AsyncRead + Unpin + Send,
    {
        let reader_addr = BytesCreate::new(self.cache).write(reader).await?;
        let kvs = vec![(Key::from("bytes"), Value::from(reader_addr))];
        // refimpl::Create::new(self.storage)
        //     .with_vec(vec![(key.into(), value.into())])
        //     .await?
        todo!()
    }
}
#[derive(Debug, Clone)]
pub struct Metadata {
    pub file_name: String,
    /* TODO: enable once Value/Scalar has Time.
     * pub updated_at: Time, */
}
impl<C, W> AsWorkspaceRef for File<'_, C, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn as_workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<C, W> AsCacheRef for File<'_, C, W>
where
    C: CacheRead + CacheWrite,
{
    type Cache = C;
    fn as_cache_ref(&self) -> &Self::Cache {
        &self.cache
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::core::Fixity};
    #[tokio::test]
    async fn write() {
        let (c, w) = Fixity::memory().into_cw();
        let f = File::new(&c, &w, Path::new());
    }
}
