use {
    super::{Error, StorageRead, StorageWrite},
    crate::Addr,
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};
#[derive(Debug, Default, Clone)]
pub struct Memory(Arc<Mutex<HashMap<Addr, Vec<u8>>>>);
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
    async fn read<A, W>(&self, addr: A, mut w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let addr = addr.as_ref().clone();
        let r = {
            let store = self.0.lock().map_err(|err| Error::Unhandled {
                message: format!("unable to acquire storage lock: {0}", err),
            })?;
            store
                .get(&addr)
                .ok_or(Error::NotFound { addr })?
                // cloning for simplicity, since this is a test focused storage impl.
                .clone()
        };
        w.write_all(&r).await?;
        Ok(r.len() as u64)
    }
}
#[async_trait::async_trait]
impl StorageWrite for Memory {
    async fn write<A, R>(&self, addr: A, mut r: R) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        R: AsyncRead + Unpin + Send,
    {
        let addr_ref = addr.as_ref();
        let addr = addr_ref.clone();
        let mut b = Vec::new();
        r.read_to_end(&mut b).await.map_err(|err| Error::IoHash {
            hash: addr_ref.clone().long(),
            err,
        })?;
        let len = b.len();
        self.0
            .lock()
            .map_err(|err| Error::Unhandled {
                message: format!("unable to acquire store lock: {0}", err),
            })?
            .insert(addr, b);
        Ok(len as u64)
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    #[tokio::test]
    async fn io() {
        let mem = Memory::default();
        let key = Addr::hash("foo".as_bytes());
        let io_in = "bar".to_owned();
        mem.write_string(key.clone(), io_in.clone()).await.unwrap();
        let io_out = mem.read_string(key).await.unwrap();
        assert_eq!(io_out, io_in);
    }
}
