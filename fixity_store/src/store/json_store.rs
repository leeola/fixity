use {
    super::{Cid, Error, Repr, Store},
    multihash::MultihashDigest,
    serde::{de::DeserializeOwned, Serialize},
    std::{
        collections::HashMap,
        hash::Hash,
        sync::{Arc, Mutex},
    },
};
// TODO: Back this store by an actual kv storage.
// TODO: back this by an anystore?
pub struct JsonStore<C = Cid>(Mutex<HashMap<C, Arc<[u8]>>>);
impl JsonStore {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
#[async_trait::async_trait]
impl<T, C> Store<T, C> for JsonStore<C>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    C: TryFrom<Vec<u8>> + Clone + Hash + Eq + Send + Sync,
{
    type Repr = JsonRepr<T>;
    async fn put(&self, t: T) -> Result<C, Error> {
        let buf: Vec<u8> = serde_json::to_vec(&t).unwrap();
        let addr: C = multihash::Code::Blake3_256
            .digest(&buf)
            .to_bytes()
            .try_into()
            .map_err(|_| ())?;
        self.0.lock().unwrap().insert(addr.clone(), Arc::from(buf));
        Ok(addr)
    }
    async fn get(&self, cid: &C) -> Result<Self::Repr, Error> {
        let buf = {
            let map = self.0.lock().unwrap();
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
