use {
    super::{ContentStorage, Error, MutStorage},
    crate::cid::CID_LENGTH,
    async_trait::async_trait,
    std::{
        collections::{BTreeMap, HashMap},
        hash::Hash,
        sync::{Arc, Mutex},
    },
};
/// A test focused in-memory only storage.
#[derive(Debug)]
pub struct Memory<Cid = [u8; CID_LENGTH]> {
    content: Mutex<HashMap<Cid, Arc<[u8]>>>,
    mut_: Mutex<BTreeMap<String, Arc<[u8]>>>,
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
#[async_trait]
impl<Cid> MutStorage for Memory<Cid>
where
    Cid: Send,
{
    type Value = Arc<[u8]>;
    async fn list<K>(&self, prefix: K) -> Result<Vec<String>, Error>
    where
        K: AsRef<str> + Send,
    {
        let prefix = prefix.as_ref();
        let mut_ = self.mut_.lock().unwrap();
        let matches = mut_
            // NIT: This `to_string` is quite painful, however the `range()` API
            // is quite awkward
            .range(prefix.to_string()..)
            .take_while(|(key, _)| key.starts_with(prefix))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        Ok(matches)
    }
    async fn get<K>(&self, key: K) -> Result<Self::Value, Error>
    where
        K: AsRef<str> + Send,
    {
        let lock = self.mut_.lock().unwrap();
        let buf = lock.get(key.as_ref()).unwrap();
        Ok(Arc::clone(&buf))
    }
    async fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<str> + Into<String> + Send,
        V: AsRef<[u8]> + Into<Vec<u8>> + Send,
    {
        let mut mut_ = self.mut_.lock().unwrap();
        let _ = mut_.insert(key.into(), Arc::from(value.into()));
        Ok(())
    }
}
impl<C> Default for Memory<C> {
    fn default() -> Self {
        Self {
            content: Default::default(),
            mut_: Default::default(),
        }
    }
}
