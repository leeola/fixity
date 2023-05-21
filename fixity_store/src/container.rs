use crate::{
    content_store::ContentStore,
    contentid::Cid,
    deser::{Deserialize, Serialize},
    deser_ext::DeserExt,
    store::StoreError,
};
use async_trait::async_trait;
use std::sync::Arc;

/// The combination of individual container behaviors, where and `Self` that implements [`Persist`],
/// [`Reconcile`] and [`Describe`] will also implement [`Container`].
///
/// By splitting the behaviors into sub-traits, we can automatically implement sections of
/// containers behavior in certain situations.
#[async_trait]
pub trait ContainerV4<S: ContentStore>:
    PersistContainer<S> + ReconcileContainer<S> + DescribeContainer
{
}
impl<T, S> ContainerV4<S> for T
where
    S: ContentStore,
    T: PersistContainer<S> + ReconcileContainer<S> + DescribeContainer,
{
}
/// Container [default](DefaultContainer) and save/open behavior of a [`Container`].
#[async_trait]
pub trait PersistContainer<S: ContentStore>: Send + DefaultContainer<S> {
    async fn open(store: &Arc<S>, cid: &Cid) -> Result<Self, StoreError>;
    async fn save(&mut self, store: &Arc<S>) -> Result<Cid, StoreError>;
    async fn save_with_cids(
        &mut self,
        store: &Arc<S>,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError>;
}
#[async_trait]
impl<T, S> PersistContainer<S> for T
where
    T: Send + Sync + DefaultContainer<S> + Serialize + Deserialize,
    S: ContentStore,
{
    async fn open(store: &Arc<S>, cid: &Cid) -> Result<Self, StoreError> {
        store.get_owned_unchecked(cid).await
    }
    async fn save(&mut self, store: &Arc<S>) -> Result<Cid, StoreError> {
        store.put(self).await
    }
    async fn save_with_cids(
        &mut self,
        store: &Arc<S>,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        self.save_with_cids(store, cids_buf).await
    }
}
/// Like [`Default`], but with a store reference to keep as needed.
pub trait DefaultContainer<S>: Sized {
    /// Like [`Default::default`], but with a store reference to keep as needed.
    fn default_container(store: &Arc<S>) -> Self;
}
impl<T, S> DefaultContainer<S> for T
where
    T: Default,
{
    fn default_container(_: &Arc<S>) -> Self {
        Self::default()
    }
}
/// Behavior for reconciling and identifying differences between containers, primarily assuming the
/// container is CRDT-like.
///
/// Non-CRDT containers are largely going to fail these methods.
//
// TODO: change error to ReconcileError to allow for finer grained reporting on common classes of
// errors.
#[async_trait]
pub trait ReconcileContainer<S: ContentStore>: Sized + Send {
    async fn merge(&mut self, store: &Arc<S>, other: &Cid) -> Result<(), StoreError>;
    // TODO: Probably convert the return value to a `type Diff;`, to allow for container impls to
    // return a different type where that makes sense.
    async fn diff(&mut self, store: &Arc<S>, other: &Cid) -> Result<Self, StoreError>;
}
#[async_trait]
impl<S> ReconcileContainer<S> for String
where
    S: ContentStore,
{
    async fn merge(&mut self, _: &Arc<S>, _: &Cid) -> Result<(), StoreError> {
        Err(StoreError::UnmergableType)
    }
    async fn diff(&mut self, _: &Arc<S>, _: &Cid) -> Result<Self, StoreError> {
        // NIT: Maybe not true for simple types, but we'd have to pick an algo to diff a string
        // and assuming one seems wrong.
        Err(StoreError::UndiffableType)
    }
}
// #[macro_export]
// macro_rules! impl_not_reconcilable {
//     ( $type:ty ) => {
//         {
//             let mut p = Path::new();
//             $(
//                 p.push_map($x);
//             )*
//             p
//         }
//     };
// }
/// Describe the type signature of the container and any [de]serialized types the container writes
/// to the store.
pub trait DescribeContainer {
    fn description() -> ContainerDescription;
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ContainerDescription {
    pub name: &'static str,
    // NIT: Ideally this would be a const array, but nesting them is awkward (maybe impossible?) in
    // the current type system without entirely obfuscating a standard type like
    // ContainerDescription.
    pub params: Vec<ContainerDescription>,
}
impl DescribeContainer for String {
    fn description() -> ContainerDescription {
        ContainerDescription {
            name: std::any::type_name::<Self>(),
            params: Default::default(),
        }
    }
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
