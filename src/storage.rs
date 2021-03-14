pub mod fs;
pub mod memory;
pub use {fs::Fs, memory::Memory};

use {
    crate::Addr,
    tokio::io::{self, AsyncRead, AsyncWrite},
};

pub trait Storage: StorageRead + StorageWrite {}
impl<T> Storage for T where T: StorageRead + StorageWrite {}
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait StorageRead: Sync {
    async fn read<A, W>(&self, addr: A, w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        W: AsyncWrite + Unpin + Send;

    async fn read_string<A>(&self, addr: A) -> Result<String, Error>
    where
        A: AsRef<Addr> + 'static + Send,
    {
        let mut buf = Vec::new();
        self.read(addr, &mut buf).await?;
        let s = std::str::from_utf8(&buf)?.to_owned();
        Ok(s)
    }
}
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait StorageWrite: Sync {
    async fn write<A, R>(&self, addr: A, r: R) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        R: AsyncRead + Unpin + Send;

    /// A helper to write the provided String into storage.
    async fn write_string<A>(&self, addr: A, s: String) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
    {
        let b = s.as_bytes();
        self.write(addr, &*b).await
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{message}`")]
    Unhandled { message: String },
    #[error("hash `{addr}` not found")]
    NotFound { addr: Addr },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("hash `{hash}` io error: {err}")]
    IoHash { hash: String, err: io::Error },
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("hash `{hash}` io error: {err}")]
    Utf8Hash {
        hash: String,
        err: std::str::Utf8Error,
    },
}
impl Error {
    /// Whether or not the error is the `Error::NotFound` variant.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }
}
/// A helper trait to allow a single `T` to return references to both a `Workspace` and
/// a `Storage`.
///
/// See [`Commit`](crate::Commit) for example usage.
pub trait AsStorageRef {
    type Storage: Storage;
    fn as_storage_ref(&self) -> &Self::Storage;
}
// A NOOP Storage impl.
#[async_trait::async_trait]
impl StorageRead for () {
    async fn read<A, W>(&self, _: A, _: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        W: AsyncWrite + Unpin + Send,
    {
        Ok(0)
    }
}
// A NOOP Storage impl.
#[async_trait::async_trait]
impl StorageWrite for () {
    async fn write<A, R>(&self, _: A, _: R) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        R: AsyncRead + Unpin + Send,
    {
        Ok(0)
    }
}
