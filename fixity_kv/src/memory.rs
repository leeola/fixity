use {
    super::{Error, Flush, Read, Stage, Write},
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
};
#[derive(Debug, Default, Clone)]
pub struct Memory(Arc<Mutex<HashMap<Vec<u8>, Arc<[u8]>>>>);
impl Memory {
    pub fn new() -> Self {
        Self::default()
    }
}
#[async_trait::async_trait]
impl Read for Memory {
    type Bytes = Arc<[u8]>;

    async fn exists<K>(&self, k: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]> + Send,
    {
        let exists = self
            .0
            .lock()
            .map_err(|err| format!("unable to acquire memory store lock: {0}", err))?
            .contains_key(k.as_ref());
        Ok(exists)
    }
    async fn read<K>(&self, k: K) -> Result<Self::Bytes, Error>
    where
        K: AsRef<[u8]> + Send,
    {
        let inner = self
            .0
            .lock()
            .map_err(|err| format!("unable to acquire memory store lock: {0}", err))?;
        let k = k.as_ref();
        let v = inner.get(k).ok_or(Error::NotFound)?;
        Ok(Arc::clone(&v))
    }
}
#[async_trait::async_trait]
impl Write for Memory {
    async fn write<K, V>(&self, k: K, v: V) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
        V: AsRef<[u8]> + Send,
    {
        self.0
            .lock()
            .map_err(|err| format!("unable to acquire memory store lock: {0}", err))?
            .insert(k.as_ref().to_owned(), Arc::from(v.as_ref().to_owned()));
        Ok(())
    }
}
#[async_trait::async_trait]
impl Stage for Memory {
    type Stage = MemStage;
    async fn stage(&self) -> Result<Self::Stage, Error> {
        Ok(MemStage {
            stage: Memory::new(),
            dst: Memory(Arc::clone(&self.0)),
        })
    }
}
#[derive(Debug, Default)]
pub struct MemStage {
    stage: Memory,
    dst: Memory,
}
#[async_trait::async_trait]
impl Flush for MemStage {
    async fn flush(&self) -> Result<(), Error> {
        let mut stage = self
            .stage
            .0
            .lock()
            .map_err(|err| format!("unable to acquire memory store lock: {0}", err))?;
        let mut dst = self
            .dst
            .0
            .lock()
            .map_err(|err| format!("unable to acquire memory store lock: {0}", err))?;
        for (k, v) in stage.drain() {
            dst.insert(k, v);
        }
        Ok(())
    }
}
#[async_trait::async_trait]
impl Write for MemStage {
    async fn write<K, V>(&self, k: K, v: V) -> Result<(), Error>
    where
        K: AsRef<[u8]> + Send,
        V: AsRef<[u8]> + Send,
    {
        self.stage.write(k, v).await
    }
}
