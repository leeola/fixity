use crate::{
    content_store::ContentStore,
    contentid::Cid,
    deser_store::deser_store_v4::{Deserialize, Serialize},
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
    DeserExt,
};
use async_trait::async_trait;
use std::sync::Arc;

/// The combination of individual container behaviors, where and `Self` that implements [`Persist`],
/// [`Reconcile`] and [`Describe`] will also implement [`Container`].
///
/// By splitting the behaviors into sub-traits, we can automatically implement sections of
/// containers behavior in certain situations.
#[async_trait]
pub trait ContainerV4<S: ContentStore>: PersistContainer<S> + ReconcileContainer<S>
// TODO: Re-enable container description once the TypeDescription impl is better defined.
//  + DescribeContainer
{
}
impl<T, S> ContainerV4<S> for T
where
    S: ContentStore,
    T: PersistContainer<S> + ReconcileContainer<S>, // + DescribeContainer,
{
}
/// Container [default](DefaultContainer) and IO behavior of a [`Container`].
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
/// Describe the type signature of the container and any [de]serialized types the container writes
/// to the store.
pub trait DescribeContainer {
    fn container_desc() -> ValueDesc;
    /// A description of the [de]serialized type(s) that this container manages.
    ///
    /// Used to determine / validate Fixity repository types.
    ///
    /// This is in contrast to the `Container: TypeDescription` bound for `Self`,
    /// which describes the `Container` itself - which may or may not be what is written
    /// to stores.
    fn deser_desc() -> ValueDesc;
}
// NIT: Not convinced this is the correct implementation. The attempt is an assumption over
// conditions of Self where if true, then the Self is the same value being [de]serialized. Given the
// right condition it seems sane to assume Self is the same thing being written, but perhaps there's
// edge cases where this will be problematic/annoying/incorrect.
impl<T> DescribeContainer for T
where
    T: TypeDescription + Serialize + Deserialize,
{
    fn container_desc() -> ValueDesc {
        Self::type_desc()
    }
    fn deser_desc() -> ValueDesc {
        Self::type_desc()
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
