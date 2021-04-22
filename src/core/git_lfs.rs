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
    std::{fmt, io::Cursor, mem, ops::Bound},
    tokio::io::{self, AsyncRead, AsyncWrite},
};
/// The only version this impl uses, currently.
const VERSION: &str = "https://git-lfs.github.com/spec/v1";
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
    pub async fn read<Writer>(&self, oid: String, w: Writer) -> Result<Option<u64>, Error>
    where
        C: CacheRead,
        W: Workspace,
        Writer: AsyncWrite + Unpin + Send,
    {
        todo!()
    }
    pub async fn write<Reader>(&self, mut r: Reader) -> Result<(Addr, Pointer), Error>
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
        let (bytes_len, checksum, bytes_addr) = {
            let mut buf = Vec::new();
            let len = io::copy(&mut r, &mut buf).await?;
            let addr = BytesCreate::new(self.cache)
                .write(Cursor::new(&buf))
                .await?;
            (
                len,
                // A separate checksum formatted as git-lfs expects it normally.
                // this is usually `"sha256:hash"`
                lfs_checksum(&buf),
                addr,
            )
        };
        let new_map_addr = if let Some(map_addr) = old_map_addr {
            let kvs = vec![(
                Key(Value::String(checksum.clone())),
                MapChange::Insert(Value::Addr(bytes_addr)),
            )];
            MapUpdate::new(self.cache, map_addr).with_vec(kvs).await?
        } else {
            let kvs = vec![(
                Key(Value::String(checksum.clone())),
                Value::Addr(bytes_addr),
            )];
            MapCreate::new(self.cache).with_vec(kvs).await?
        };
        let new_staged_content = self
            .path
            .update(self.cache, resolved_path, new_map_addr)
            .await?;
        workspace_guard.stage(new_staged_content.clone()).await?;
        Ok((
            new_staged_content,
            Pointer {
                version: VERSION.to_owned(),
                oid: checksum,
                size: bytes_len,
            },
        ))
    }
}
/// The data to construct a Git LFS pointer file.
///
/// See also: https://github.com/git-lfs/git-lfs/blob/main/docs/spec.md#the-pointer
#[derive(Debug)]
pub struct Pointer {
    /// Per spec documentation: version is a URL that identifies the pointer file spec. Parsers
    /// MUST use simple string comparison on the version, without any URL parsing or normalization.
    /// It is case sensitive, and %-encoding is discouraged.
    pub version: String,
    /// Per spec documentation: oid tracks the unique object id for the file, prefixed by its
    /// hashing method: {hash-method}:{hash}. Currently, only sha256 is supported. The hash is
    /// lower case hexadecimal.
    pub oid: String,
    /// Per spec documentation: size is in bytes.
    pub size: u64,
}
fn lfs_checksum(buf: &[u8]) -> String {
    format!("sha256:{}", sha256::digest_bytes(buf))
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::core::Fixity, std::io::Cursor};
    #[tokio::test]
    async fn basic_write_read() {
        let (c, w) = Fixity::memory().into_cw();
        let git_lfs = GitLfs::new(&c, &w, Path::new());
        let pointer = git_lfs.write(Cursor::new(b"foo")).await.unwrap();
        dbg!(&pointer);
    }
}
