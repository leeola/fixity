use std::sync::Arc;

use crate::{
    content_store::ContentStore,
    contentid::Cid,
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
};
use async_trait::async_trait;

#[async_trait]
pub trait ContainerV4<S: ContentStore>: Sized + Send + TypeDescription {
    /// A description of the [de]serialized type(s) that this container manages.
    ///
    /// Used to determine / validate Fixity repository types.
    ///
    /// This is in contrast to the `Container: TypeDescription` bound for `Self`,
    /// which describes the `Container` itself - which may or may not be what is written
    /// to stores.
    fn deser_type_desc() -> ValueDesc;
    fn new_container(store: &Arc<S>) -> Self;
    async fn open(store: &Arc<S>, cid: &Cid) -> Result<Self, StoreError>;
    async fn save(&mut self, store: &Arc<S>) -> Result<Cid, StoreError>;
    async fn save_with_cids(
        &mut self,
        store: &Arc<S>,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError>;
    async fn merge(&mut self, store: &Arc<S>, other: &Cid) -> Result<(), StoreError>;
    // TODO: Probably convert the return value to a `type Diff;`, to allow for container impls to
    // return a different type where that makes sense.
    async fn diff(&mut self, store: &Arc<S>, other: &Cid) -> Result<Self, StoreError>;
    // TODO: Method to report contained Cids and/or Containers to allow correct syncing of a
    // Container and all the cids within it.
}
// // TODO: revisit before new usage. Make a v4 version, any desired changes, etc.
// // Notably i'd really like to make the `Ref` type borrowed from whatever returned value
// `open_ref` // is.
// #[async_trait]
// pub trait ContainerRef<S: ContentStore>: ContainerV4<S> {
//     type Ref: ContainerRefInto<Self>;
//     type DiffRef: ContainerRefInto<Self>;
//     async fn open_ref(store: &S, cid: &Cid) -> Result<Self::Ref, StoreError>;
//     async fn diff_ref(&mut self, store: &S, other: &Cid) -> Result<Self::DiffRef, StoreError>;
// }
// NIT: Infallible conversions were making `TryInto` awkward for `Ref` and `DiffRef` on
// `ContainerRef`, so this trait fills that role without the infallible issues.
// I must be misunderstanding how to deal with Infallible `TryInto`'s easily, while
// also putting bounds on the associated `TryInto::Error` type.
//
// Or perhaps it's just awkward because associated type bounds don't exist yet.
pub trait ContainerRefInto<Owned> {
    type Error: Into<StoreError>;
    fn container_ref_into(self) -> Result<Owned, Self::Error>;
}
impl<Owned> ContainerRefInto<Owned> for Owned {
    type Error = StoreError;
    fn container_ref_into(self) -> Result<Owned, Self::Error> {
        Ok(self)
    }
}
