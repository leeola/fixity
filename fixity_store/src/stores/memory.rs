use crate::{
    byte_store::{ByteStore, ByteStoreError},
    contentid::NewContentId,
};
use async_trait::async_trait;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

/// A (currently) test focused in-memory only storage.
#[derive(Debug)]
pub struct Memory<Cid> {
    // TODO: change to faster concurrency primitives. At
    // the very least, RwLock instead of Mutex.
    bytes: Mutex<HashMap<Cid, Arc<[u8]>>>,
    mut_: Mutex<BTreeMap<String, Arc<[u8]>>>,
}
#[async_trait]
impl<Cid> ByteStore<Cid> for Memory<Cid>
where
    Cid: NewContentId,
{
    type Bytes = Arc<[u8]>;
    async fn exists(&self, cid: &Cid) -> Result<bool, ByteStoreError> {
        Ok(self.bytes.lock().unwrap().contains_key(cid))
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ByteStoreError> {
        let lock = self.bytes.lock().unwrap();
        let buf = lock.get(cid).unwrap();
        Ok(Arc::clone(&buf))
    }
    async fn write_unchecked(&self, cid: &Cid, bytes: Vec<u8>) -> Result<(), ByteStoreError> {
        let mut lock = self.bytes.lock().unwrap();
        let _ = lock.insert(cid.clone(), Arc::from(bytes));
        Ok(())
    }
}
