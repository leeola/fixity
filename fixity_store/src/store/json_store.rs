use {
    super::{
        cid::{CidHasher, Hashers},
        Error, Repr, Store,
    },
    serde::{de::DeserializeOwned, Serialize},
    std::{
        collections::HashMap,
        hash::Hash,
        sync::{Arc, Mutex},
    },
};
// TODO: Back this store by an actual kv storage.
// TODO: back this by an anystore?
pub struct JsonStore<H = Hashers>
where
    H: CidHasher,
{
    hasher: H,
    data: Mutex<HashMap<H::Cid, Arc<[u8]>>>,
}
impl<H> JsonStore<H>
where
    H: CidHasher,
{
    pub fn new() -> Self
    where
        H: Default,
    {
        Self {
            hasher: Default::default(),
            data: Mutex::new(HashMap::new()),
        }
    }
}
#[async_trait::async_trait]
impl<T, H> Store<T, H> for JsonStore<H>
where
    H: CidHasher + Sync,
    H::Cid: Clone + Hash + Eq + Send + Sync,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    type Repr = JsonRepr<T>;
    async fn put(&self, t: T) -> Result<H::Cid, Error> {
        let buf: Vec<u8> = serde_json::to_vec(&t).unwrap();
        let cid = self.hasher.hash(&buf);
        self.data
            .lock()
            .unwrap()
            .insert(cid.clone(), Arc::from(buf));
        Ok(cid)
    }
    async fn get(&self, cid: &H::Cid) -> Result<Self::Repr, Error> {
        let buf = {
            let map = self.data.lock().unwrap();
            Arc::clone(&map.get(cid).unwrap())
        };
        Ok(JsonRepr {
            value: serde_json::from_slice(buf.as_ref()).unwrap(),
        })
    }
}
pub struct JsonRepr<T> {
    value: T,
}
impl<T> Repr for JsonRepr<T>
where
    T: Clone,
{
    type Owned = T;
    type Borrow = T;
    fn repr_to_owned(&self) -> Result<Self::Owned, Error> {
        Ok(self.value.clone())
    }
    fn repr_borrow(&self) -> Result<&Self::Borrow, Error> {
        Ok(&self.value)
    }
}
