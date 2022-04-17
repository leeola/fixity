pub mod memory;
pub mod object;
pub mod object_ref;
use std::{ops::Deref, str};
pub use {memory::Memory, object::ObjectKv, object_ref::ObjectRefKv};

#[async_trait::async_trait]
pub trait Read: Sync {
    type Bytes: Deref<Target = [u8]>;

    async fn exists<K>(&self, k: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]> + Send;

    async fn read<K>(&self, k: K) -> Result<Self::Bytes, Error>
    where
        K: AsRef<[u8]> + Send;

    async fn read_string<K>(&self, k: K) -> Result<String, Error>
    where
        K: AsRef<[u8]> + Send,
    {
        let buf = self.read(k).await?;
        let s = str::from_utf8(&buf.as_ref())
            .map_err(|err| Error::ReadIntoString(Box::new(err)))?
            .to_owned();
        Ok(s)
    }
}
#[async_trait::async_trait]
pub trait Write: Sync {
    async fn write<K, V>(&self, k: K, v: V) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
        // TODO: Probably into an IVec or something?
        V: AsRef<[u8]> + Send;

    /// A helper to write the provided String into storage.
    async fn write_string<K>(&self, addr: K, s: String) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
    {
        self.write(addr, s.as_bytes()).await
    }
}
#[async_trait::async_trait]
pub trait Stage: Sync {
    type Stage: Flush;
    async fn stage(&self) -> Result<Self::Stage, Error>;
}
#[async_trait::async_trait]
pub trait Flush: Write {
    async fn flush(&self) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("key not found")]
    NotFound,
    #[error("read into string: {0}")]
    ReadIntoString(Box<dyn std::error::Error>),
    #[error("unknown: `{0}`")]
    Unknown(Box<dyn std::error::Error>),
}
impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Unknown(s.into())
    }
}
impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Unknown(s.into())
    }
}
