use {
    super::{Addr, Error, Store, StoreRef},
    multihash::MultihashDigest,
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::{
        borrow::Borrow,
        collections::HashMap,
        hash::Hash,
        marker::PhantomData,
        sync::{Arc, Mutex},
    },
};
pub struct JsonRef<T> {
    buf: Arc<[u8]>,
    _phantom: PhantomData<T>,
}
// TODO: Back this store by an actual kv storage.
pub struct JsonStore<A = Addr>(Mutex<HashMap<A, Arc<[u8]>>>);
impl JsonStore {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
#[async_trait::async_trait]
impl<T, A> Store<T, A> for JsonStore<A>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
    A: TryFrom<Vec<u8>> + Clone + Hash + Eq + Send + Sync,
{
    type Ref = JsonRef<T>;
    async fn put(&self, t: T) -> Result<A, Error> {
        let buf: Vec<u8> = serde_json::to_vec(&t).unwrap();
        let addr: A = multihash::Code::Blake3_256
            .digest(&buf)
            .to_bytes()
            .try_into()
            .map_err(|_| ())?;
        self.0.lock().unwrap().insert(addr.clone(), Arc::from(buf));
        Ok(addr)
    }
    async fn get(&self, cid: &A) -> Result<Self::Ref, Error> {
        let buf = {
            let map = self.0.lock().unwrap();
            Arc::clone(&map.get(cid).unwrap())
        };
        Ok(JsonRef {
            buf,
            _phantom: PhantomData,
        })
    }
}
impl<T> StoreRef<T> for JsonRef<T>
where
    T: DeserializeOwned,
{
    type Repr = T;
    fn repr_to_owned(&self) -> Result<T, Error> {
        use std::ops::Deref;
        let t: T = serde_json::from_slice(self.buf.deref()).unwrap();
        Ok(t)
    }
    fn repr_borrow<'a, U: ?Sized>(&'a self) -> Result<&'a U, Error>
    where
        Self::Repr: Borrow<U>,
        U: Deserialize<'a>,
    {
        todo!()
    }
}
