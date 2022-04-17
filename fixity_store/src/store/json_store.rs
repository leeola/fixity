use {
    super::{Error, Repr, Store},
    crate::{
        cid::{ContentHasher, Hasher},
        storage::{memory::Memory, ContentStorage},
    },
    async_trait::async_trait,
    serde::{de::DeserializeOwned, Serialize},
};
// TODO: Cache the serialized values, to reduce deserialization cost.
pub struct JsonStore<Storage, H = Hasher> {
    hasher: H,
    storage: Storage,
}
impl<Storage, H> JsonStore<Storage, H> {
    pub fn new(storage: Storage) -> Self
    where
        H: ContentHasher + Default,
        Storage: ContentStorage<H::Cid>,
    {
        Self {
            hasher: Default::default(),
            storage,
        }
    }
}
impl JsonStore<Memory> {
    pub fn memory() -> Self {
        Self::new(Default::default())
    }
}
#[async_trait]
impl<T, S, H> Store<T, H> for JsonStore<S, H>
where
    S: ContentStorage<H::Cid>,
    S::Content: From<Vec<u8>>,
    H: ContentHasher,
    H::Cid: Copy,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    type Repr = JsonRepr<T>;
    async fn put(&self, t: T) -> Result<H::Cid, Error> {
        // FIXME: prototype unwraps.
        let buf: Vec<u8> = serde_json::to_vec(&t).unwrap();
        let cid = self.hasher.hash(&buf);
        self.storage.write_unchecked(cid, buf).await?;
        Ok(cid)
    }
    async fn get(&self, cid: &H::Cid) -> Result<Self::Repr, Error> {
        // FIXME: prototype unwraps.
        let buf = self.storage.read_unchecked(cid).await?;
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
