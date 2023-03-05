use crate::{content_store::ContentStore, contentid::NewContentId};

pub trait ReprRef<Repr>: Sized {
    type Ref<'a>;
}
// store can deserialize the combo params
pub trait DeserializeRepr<Deser, T, Repr>
where
    T: ReprRef<Repr>,
{
    fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError>;
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError>;
}

#[async_trait]
pub trait DeserStoreV3<Cid: NewContentId>: ContentStore<Cid> {
    type Deser;
    async fn get<T>(&self, cid: &Cid) -> Result<Repr<T, Deser>, StoreError>
    where
        Self: RefRepr<Self::Deser, RefRepr>;
    async fn put<T>(&self, t: &T) -> Result<Cid, StoreError>
    where
        T: Serialize<Deser> + Send + Sync;
    async fn put_with_cids<T>(&self, t: &T, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>
    where
        T: Serialize<Deser> + Send + Sync;
}

pub trait Deserialize<Repr, T>
where
    T: ReprRef<Repr>,
{
}

impl<S, Cid, Repr, T> Deserialize<Repr, T> for S
where
    S: DeserStoreV3<Cid>,
    S: DeserializeRepr<S::Deser, Repr, T>,
{
}
