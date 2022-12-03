use crate::{
    contentid::NewContentId,
    deser::{Deserialize, Serialize},
    deser_store::DeserStore,
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
    Store,
};
use async_trait::async_trait;

#[async_trait]
pub trait NewContainer<Deser, Cid: NewContentId>: Sized + Send + TypeDescription {
    /// A description of the [de]serialized type(s) that this container manages.
    ///
    /// Used to determine / validate Fixity repository types.
    ///
    /// This is in contrast to the `Container: TypeDescription` bound for `Self`,
    /// which describes the `Container` itself - which may or may not be what is written
    /// to stores.
    fn deser_type_desc() -> ValueDesc;
    fn new_container<S: DeserStore<Deser, Cid>>(store: &S) -> Self;
    async fn open<S: DeserStore<Deser, Cid>>(store: &S, cid: &Cid) -> Result<Self, StoreError>;
    async fn save<S: DeserStore<Deser, Cid>>(&mut self, store: &S) -> Result<Cid, StoreError>;
    async fn save_with_cids<S: DeserStore<Deser, Cid>>(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError>;
    async fn merge<S: DeserStore<Deser, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<(), StoreError>;
    async fn diff<S: DeserStore<Deser, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<Self, StoreError>;
}
#[async_trait]
pub trait ContainerRef<Deser, Cid: NewContentId>: NewContainer<Deser, Cid> {
    type Ref: TryInto<Self, Error = StoreError>;
    type DiffRef: TryInto<Self, Error = StoreError>;
    async fn open_ref<S: DeserStore<Deser, Cid>>(
        store: &S,
        cid: &Cid,
    ) -> Result<Self::Ref, StoreError>;
    async fn diff_ref<S: DeserStore<Deser, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<Self::DiffRef, StoreError>;
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
