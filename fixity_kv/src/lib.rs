pub mod memory;
pub use memory::Memory;
use {std::str, tokio::io::AsyncRead};

#[async_trait::async_trait]
pub trait Read: Sync {
    type Buf: AsRef<[u8]>;

    async fn exists<K>(&self, k: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]> + Send;

    async fn read<K>(&self, k: K) -> Result<Self::Buf, Error>
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
    type Stage: WriteStg;
    async fn stage(&self) -> Result<Self::Stage, Error>;
}
#[async_trait::async_trait]
pub trait WriteStg: Sync {
    async fn write<K, V>(&self, k: K, v: V) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
        V: AsRef<[u8]> + Send;

    /// A helper to write the provided String into storage.
    async fn write_string<K>(&self, addr: K, s: String) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
    {
        self.write(addr, s.as_bytes()).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("read into string: {0}")]
    ReadIntoString(Box<dyn std::error::Error>),
    #[error("unknown: `{0}`")]
    Unknown(Box<dyn std::error::Error>),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
