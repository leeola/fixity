use {
    super::{Addr, Error, Repr, ReprBorrow, Store},
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
pub struct JsonRepr<T> {
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
    type Repr = JsonRepr<T>;
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
    async fn get(&self, cid: &A) -> Result<Self::Repr, Error> {
        let buf = {
            let map = self.0.lock().unwrap();
            Arc::clone(&map.get(cid).unwrap())
        };
        Ok(JsonRepr {
            buf,
            _phantom: PhantomData,
        })
    }
}
impl<T> Repr<T> for JsonRepr<T>
where
    T: DeserializeOwned,
{
    fn repr_to_owned(&self) -> Result<T, Error> {
        let t: T = serde_json::from_slice(self.buf.as_ref()).unwrap();
        Ok(t)
    }
}
// impl<'a, T, Borrowed> ReprBorrow<Borrowed> for JsonRepr<T>
// where
//     T: Borrow<Borrowed>,
//     Borrowed: ?Sized,
//     Borrowed: Deserialize<'a>,
//     Self: 'a,
// {
//     fn borrow_from_repr(&self) -> Result<&Borrowed, Error> {
//         let b: Borrowed = serde_json::from_slice(self.buf.as_ref()).unwrap();
//         todo!()
//     }
// }
pub trait ReprBorrowRef {
    fn ref_from_repr<'a, Borrowed: 'a>(&'a self) -> Result<Borrowed, Error>
    where
        Self: ReprBorrowRefBlah<'a, Borrowed>,
    {
        ReprBorrowRefBlah::ref_from_repr_blah(self)
    }
}
impl<T> ReprBorrowRef for JsonRepr<T> {}
pub trait ReprBorrowRefBlah<'a, Borrowed: 'a> {
    fn ref_from_repr_blah(&'a self) -> Result<Borrowed, Error>;
}
impl<'a, T, Borrowed> ReprBorrowRefBlah<'a, Borrowed> for JsonRepr<T>
where
    Borrowed: 'a + Deserialize<'a>,
{
    fn ref_from_repr_blah(&'a self) -> Result<Borrowed, Error> {
        let b: Borrowed = serde_json::from_slice(self.buf.as_ref()).unwrap();
        Ok(b)
    }
}
