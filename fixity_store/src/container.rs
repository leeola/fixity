use crate::{
    deser::{Deserialize, Serialize},
    store::StoreError,
    Store,
};
use async_trait::async_trait;

#[async_trait]
pub trait NewContainer<'s, S>: Sized + Send + 's
where
    S: Store,
{
    type Ref: TryInto<Self, Error = StoreError>;
    async fn open(store: &'s S, cid: &S::Cid) -> Result<Self, StoreError>;
    async fn open_ref(store: &'s S, cid: &S::Cid) -> Result<Self::Ref, StoreError>;
    async fn save(&mut self, store: &'s S) -> Result<S::Cid, StoreError>;
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<S::Cid>,
    ) -> Result<(), StoreError>;
}

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

    // NOTE: other:Cid looks good, but being able to describe a diff should/could
    // be baked in.. somewhere.
    //
    // async fn merge(&mut self, other: &S::Cid) -> Result<(), StoreError>;
    // async fn merge(&mut self, other: &Self) -> Result<(), StoreError>;

    // TODO: how do we generically describe a diff? Could look to some diffing libraries for
    // inspiration?
    //
    // It's possible we generate another Self of the diff, for a verbose thing at least,
    // which lets the Container type itself present itself however. Eg a diff of counters
    // might print as `5`, a diff of SQL could itself be queryable or  CSV-able, etc.
    //
    // Seems possibly heavy, not sure if we'd want a "lean" diff presentation or not.
    //
    // async fn diff(&self, other: &S::Cid) -> Result<???, StoreError>;
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
