// pub mod memory;
// pub use memory::Memory;
pub mod fs;

use {
    async_trait::async_trait,
    tokio::io::{self, AsyncRead, AsyncWrite},
};

pub trait Storage: StorageRead + StorageWrite {}
impl<T> Storage for T where T: StorageRead + StorageWrite {}

#[async_trait::async_trait]
pub trait StorageRead {
    async fn read<S, W>(&self, hash: S, w: W) -> Result<(), Error>
    where
        S: AsRef<str>,
        W: AsyncWrite;

    // async fn read_string<S>(&self, hash: S) -> Result<String, Error>
    // where
    //     S: AsRef<str>,
    // {
    //     let mut buf = BufWriter::new(Vec::new());
    //     self.read(&hash, &mut buf)?;
    //     buf.flush().map_err(|err| Error::Io {
    //         hash: hash.as_ref().to_owned(),
    //         err,
    //     })?;
    //     let s = std::str::from_utf8(&buf.get_ref())
    //         .map_err(|err| Error::Utf8 {
    //             hash: hash.as_ref().to_owned(),
    //             err,
    //         })?
    //         .to_owned();
    //     Ok(s)
    // }
}

#[async_trait::async_trait]
pub trait StorageWrite {
    async fn write<S, R>(&self, hash: S, r: R) -> Result<usize, Error>
    where
        S: AsRef<str> + Send,
        R: AsyncRead + Send;

    // async fn write_string<S>(&self, hash: S, s: String) -> Result<usize, Error>
    // where
    //     S: AsRef<str> + Send,
    // {
    //     let b = s.as_bytes();
    //     let len = b.len();
    //     self.write(hash, &*b).await?;
    //     Ok(len)
    // }
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
    #[error("hash `{hash}` io error: {err}")]
    Utf8 {
        hash: String,
        err: std::str::Utf8Error,
    },
}
