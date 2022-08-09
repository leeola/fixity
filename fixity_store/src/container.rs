use crate::{
    deser::{Deserialize, Serialize},
    store::StoreError,
    Store,
};
use async_trait::async_trait;

#[async_trait]
pub trait Container<S>: Sized + Send + Default
where
    S: Store,
{
    // ^ :TypeName
    // async fn type() -> TypeSig;
    // async fn serialized_type() -> TypeSig;
    async fn open(store: &S, cid: &S::Cid) -> Result<Self, StoreError>;
    async fn save(&self, store: &S) -> Result<S::Cid, StoreError>;
    async fn save_with_cids(&self, store: &S, cids_buf: &mut Vec<S::Cid>)
        -> Result<(), StoreError>;
}
#[async_trait]
impl<T, S> Container<S> for T
where
    S: Store,
    T: Serialize<S::Deser> + Deserialize<S::Deser> + Send + Sync + Default,
{
    async fn open(store: &S, cid: &S::Cid) -> Result<Self, StoreError> {
        let repr = store.get::<Self>(cid).await?;
        repr.repr_to_owned()
    }
    async fn save(&self, store: &S) -> Result<S::Cid, StoreError> {
        store.put(self).await
    }
    async fn save_with_cids(
        &self,
        store: &S,
        cids_buf: &mut Vec<S::Cid>,
    ) -> Result<(), StoreError> {
        let cid = store.put(self).await?;
        cids_buf.push(cid);
        Ok(())
    }
}
