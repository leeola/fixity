use {
    super::{Error, StorageRead, StorageWrite},
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};
#[derive(Debug, Default, Clone)]
pub struct Memory(Arc<Mutex<HashMap<String, String>>>);
impl Memory {
    pub fn new() -> Self {
        Self::default()
    }
}
#[cfg(test)]
impl std::cmp::PartialEq<Self> for Memory {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .lock()
            .expect("failed to lock Lhs")
            .eq(&*other.0.lock().expect("failed to lock Rhs"))
    }
}
#[async_trait::async_trait]
impl StorageRead for Memory {
    async fn read<S, W>(&self, hash: S, mut w: W) -> Result<(), Error>
    where
        S: AsRef<str> + 'static + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let hash = hash.as_ref();
        let r = {
            let store = self.0.lock().map_err(|err| Error::Unhandled {
                message: format!("unable to acquire storage lock: {0}", err),
            })?;
            store
                .get(hash)
                .ok_or_else(|| Error::NotFound {
                    hash: hash.to_owned(),
                })?
                // cloning for simplicity, since this is a test focused storage impl.
                .clone()
        };
        w.write_all(&r.as_bytes()).await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl StorageWrite for Memory {
    async fn write<S, R>(&self, hash: S, mut r: R) -> Result<u64, Error>
    where
        S: AsRef<str> + 'static + Send,
        R: AsyncRead + Unpin + Send,
    {
        let hash = hash.as_ref();
        let mut b = Vec::new();
        r.read_to_end(&mut b).await.map_err(|err| Error::IoHash {
            hash: hash.to_owned(),
            err,
        })?;
        let len = b.len();
        let s = String::from_utf8(b).map_err(|_| Error::Unhandled {
            message: format!("{} is not valid utf8", hash),
        })?;
        self.0
            .lock()
            .map_err(|err| Error::Unhandled {
                message: format!("unable to acquire store lock: {0}", err),
            })?
            .insert(hash.to_owned(), s);
        Ok(len as u64)
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    #[tokio::test]
    async fn io() {
        let mem = Memory::default();
        let key = "foo";
        let io_in = "bar".to_owned();
        mem.write_string(key, io_in.clone()).await.unwrap();
        let io_out = mem.read_string(key).await.unwrap();
        assert_eq!(io_out, io_in);
    }
}
