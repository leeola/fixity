pub mod fs;
pub mod memory;
pub use {fs::Fs, memory::Memory};

use tokio::io::{self, AsyncRead, AsyncWrite};

pub trait Storage: StorageRead + StorageWrite {}
impl<T> Storage for T where T: StorageRead + StorageWrite {}

#[async_trait::async_trait]
pub trait StorageRead: Sync {
    async fn read<S, W>(&self, hash: S, w: W) -> Result<u64, Error>
    where
        S: AsRef<str> + 'static + Send,
        W: AsyncWrite + Unpin + Send;

    async fn read_string<S>(&self, hash: S) -> Result<String, Error>
    where
        S: AsRef<str> + 'static + Send,
    {
        let mut buf = Vec::new();
        self.read(hash, &mut buf).await?;
        let s = std::str::from_utf8(&buf)?.to_owned();
        Ok(s)
    }
}

#[async_trait::async_trait]
pub trait StorageWrite: Sync {
    async fn write<S, R>(&self, hash: S, r: R) -> Result<u64, Error>
    where
        S: AsRef<str> + 'static + Send,
        R: AsyncRead + Unpin + Send;

    /// A helper to write the provided String into storage.
    async fn write_string<S>(&self, hash: S, s: String) -> Result<u64, Error>
    where
        S: AsRef<str> + 'static + Send,
    {
        let b = s.as_bytes();
        self.write(hash, &*b).await
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{message}`")]
    Unhandled { message: String },
    #[error("hash `{hash}` not found")]
    NotFound { hash: String },
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
