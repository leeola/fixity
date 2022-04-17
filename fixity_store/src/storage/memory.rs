use {
    super::{ContentStorage, Error},
    crate::cid::Cid as DefaultCid,
    async_trait::async_trait,
    std::{
        collections::HashMap,
        hash::Hash,
        sync::{Arc, Mutex},
    },
};
#[derive(Debug)]
pub struct Memory<Cid = DefaultCid> {
    content: Mutex<HashMap<Cid, Arc<[u8]>>>,
    // reflog: Arc<Mutex<HashMap<PathBuf, Arc<[u8]>>>> ,
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
    async fn write_unchecked<Content>(&self, cid: Cid, content: Content) -> Result<(), Error>
    where
        Content: Into<Self::Content> + Send + 'static,
    {
        let buf = content.into();
        let mut lock = self.content.lock().unwrap();
        let _ = lock.insert(cid, buf);
        Ok(())
    }
}
impl<C> Default for Memory<C> {
    fn default() -> Self {
        Self {
            content: Default::default(),
        }
    }
}
