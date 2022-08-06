use crate::{
    deser::{Deserialize, Serialize},
    Store,
};
use async_trait::async_trait;
pub type Error = ();
#[async_trait]
pub trait Container<S>: Sized + Send + Default
where
    S: Store,
{
    // ^ :TypeName
    // async fn type() -> TypeSig;
    // async fn serialized_type() -> TypeSig;
    async fn open(store: &S, cid: &S::Cid) -> Result<Self, Error>;
    async fn save(&self, store: &S) -> Result<S::Cid, Error>;
    async fn save_with_cids(&self, store: &S, cids_buf: &mut Vec<S::Cid>) -> Result<(), Error>;
    fn new() -> Self {
        Self::default()
    }
}
#[async_trait]
impl<T, S> Container<S> for T
where
    S: Store,
    T: Serialize<S::Deser> + Deserialize<S::Deser> + Send + Sync + Default,
{
    async fn open(store: &S, cid: &S::Cid) -> Result<Self, Error> {
        let repr = store.get::<Self>(cid).await?;
        repr.repr_to_owned()
    }
    async fn save(&self, store: &S) -> Result<S::Cid, Error> {
        store.put(self).await
    }
    async fn save_with_cids(&self, store: &S, cids_buf: &mut Vec<S::Cid>) -> Result<(), Error> {
        let cid = store.put(self).await?;
        cids_buf.push(cid);
        Ok(())
    }
}
