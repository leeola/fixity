pub mod memory;
pub use memory::Memory;
use std::io::{self, BufWriter, Read, Write};
pub trait Storage: StorageRead + StorageWrite {}
impl<T> Storage for T where T: StorageRead + StorageWrite {}
pub trait StorageRead {
    fn read<S, W>(&self, hash: S, w: W) -> Result<(), Error>
    where
        S: AsRef<str>,
        W: Write;
    fn read_string<S>(&self, hash: S) -> Result<String, Error>
    where
        S: AsRef<str>,
    {
        let mut buf = BufWriter::new(Vec::new());
        self.read(&hash, &mut buf)?;
        buf.flush().map_err(|err| Error::Io {
            hash: hash.as_ref().to_owned(),
            err,
        })?;
        let s = std::str::from_utf8(&buf.get_ref())
            .map_err(|err| Error::Utf8 {
                hash: hash.as_ref().to_owned(),
                err,
            })?
            .to_owned();
        Ok(s)
    }
}
pub trait StorageWrite {
    fn write<S, R>(&self, hash: S, r: R) -> Result<usize, Error>
    where
        S: AsRef<str>,
        R: Read;
    fn write_string<S>(&self, hash: S, s: String) -> Result<usize, Error>
    where
        S: AsRef<str>,
    {
        let b = s.as_bytes();
        let len = b.len();
        self.write(hash, &*b)?;
        Ok(len)
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{message}`")]
    Unhandled { message: String },
    #[error("hash `{hash}` not found")]
    NotFound { hash: String },
    #[error("hash `{hash}` io error: {err}")]
    Io { hash: String, err: io::Error },
    #[error("hash `{hash}` io error: {err}")]
    Utf8 {
        hash: String,
        err: std::str::Utf8Error,
    },
}
