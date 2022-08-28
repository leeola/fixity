use crate::{
    deser::{Deserialize, Serialize},
    store::StoreError,
    Store,
};
use async_trait::async_trait;

#[async_trait]
pub trait Container<'s, S>: Sized + Send + 's
where
    S: Store,
{
    // ^ :TypeName
    // async fn type() -> TypeSig;
    // async fn serialized_type() -> TypeSig;
    fn new(store: &'s S) -> Self;
    async fn open(store: &'s S, cid: &S::Cid) -> Result<Self, StoreError>;
    async fn save(&mut self, store: &'s S) -> Result<S::Cid, StoreError>;
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<S::Cid>,
    ) -> Result<(), StoreError>;
}
#[async_trait]
impl<'s, T, S> Container<'s, S> for T
where
    S: Store,
    T: Serialize<S::Deser> + Deserialize<S::Deser> + Send + Sync + Default + 's,
{
    fn new(_: &'_ S) -> Self {
        Self::default()
    }
    async fn open(store: &'s S, cid: &S::Cid) -> Result<Self, StoreError> {
        let repr = store.get::<Self>(cid).await?;
        repr.repr_to_owned()
    }
    async fn save(&mut self, store: &'s S) -> Result<S::Cid, StoreError> {
        store.put(self).await
    }
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<S::Cid>,
    ) -> Result<(), StoreError> {
        let cid = store.put(self).await?;
        cids_buf.push(cid);
        Ok(())
    }
}
