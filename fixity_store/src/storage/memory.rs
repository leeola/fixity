use {
    super::{ContentStorage, Error, MetaStorage},
    crate::cid::CID_LENGTH,
    async_trait::async_trait,
    std::{
        collections::HashMap,
        hash::Hash,
        sync::{Arc, Mutex},
    },
};
#[derive(Debug)]
pub struct Memory<Cid = [u8; CID_LENGTH]> {
    content: Mutex<HashMap<Cid, Arc<[u8]>>>,
    meta: Mutex<HashMap<(String, Cid, String), Arc<[u8]>>>,
}
#[async_trait]
impl<Cid> ContentStorage<Cid> for Memory<Cid>
where
    Cid: Hash + Eq + Send + Sync,
{
    type Content = Arc<[u8]>;
    async fn exists(&self, cid: &Cid) -> Result<bool, Error> {
        Ok(self.content.lock().unwrap().contains_key(cid))
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Content, Error> {
        let lock = self.content.lock().unwrap();
        let buf = lock.get(cid).unwrap();
        Ok(Arc::clone(&buf))
    }
    async fn write_unchecked<B>(&self, cid: Cid, bytes: B) -> Result<(), Error>
    where
        B: Into<Vec<u8>> + Send + 'static,
    {
        let mut lock = self.content.lock().unwrap();
        let bytes = bytes.into();
        let _ = lock.insert(cid, bytes.into());
        Ok(())
    }
}
impl<C> Default for Memory<C> {
    fn default() -> Self {
        Self {
            content: Default::default(),
            meta: Default::default(),
        }
    }
}
