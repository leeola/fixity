use std::ops::{Deref, DerefMut};

use crate::{
    container::ContainerV4,
    content_store::ContentStore,
    contentid::Cid,
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
};
use async_trait::async_trait;

#[async_trait]
pub trait ContainerStoreExt: ContentStore {
    fn new_container<'s, T: ContainerV4<'s, Self>>(&'s self) -> WithStore<'s, T, Self>;
    async fn open<'s, T: ContainerV4<'s, Self>>(
        &'s self,
        cid: &Cid,
    ) -> Result<WithStore<'s, T, Self>, StoreError>;
}
#[async_trait]
impl<S> ContainerStoreExt for S
where
    S: ContentStore,
{
    fn new_container<'s, T: ContainerV4<'s, Self>>(&'s self) -> WithStore<'_, T, Self> {
        WithStore {
            container: T::new_container(self),
            store: self,
        }
    }
    async fn open<'s, T: ContainerV4<'s, Self>>(
        &'s self,
        cid: &Cid,
    ) -> Result<WithStore<'s, T, Self>, StoreError> {
        let container = T::open(self, cid).await?;
        Ok(WithStore {
            container,
            store: self,
        })
    }
}
pub struct WithStore<'s, T, S> {
    container: T,
    store: &'s S,
}
impl<'s, T, S> WithStore<'s, T, S> {
    pub fn into_inner(self) -> T {
        self.container
    }
}
impl<'s, T, S> Deref for WithStore<'s, T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}
impl<'s, T, S> DerefMut for WithStore<'s, T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
/// A trait to wrap a [`Container`] and pass in an associated store to container methods..
#[async_trait]
pub trait ContainerWithStore: Send + TypeDescription {
    type Store: ContentStore;
    type Container<'s>: ContainerV4<'s, Self::Store>;
    fn deser_type_desc() -> ValueDesc;
    async fn save(&mut self) -> Result<Cid, StoreError>;
    async fn save_with_cids(&mut self, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>;
    async fn merge(&mut self, other: &Cid) -> Result<(), StoreError>;
    async fn diff<'s>(&mut self, other: &Cid) -> Result<Self::Container<'s>, StoreError>;
}
/// A glue trait to borrow a container and store, allowing for an automatic implementation of
/// [`ContainerWithStore`] for any [`Container`] that also contains a store.
pub trait AsContainerAndStore
where
    for<'s> Self: ContainerV4<'s, Self::Store> + Sync,
{
    type Store: ContentStore;
    fn as_container_store(&mut self) -> (&mut Self, &Self::Store);
}
// NIT: How does this `for T` impl not conflict with the below `for WithStore` impl? :confused:
#[async_trait]
impl<T> ContainerWithStore for T
where
    T: AsContainerAndStore,
{
    type Store = T::Store;
    type Container = T;
    fn deser_type_desc() -> ValueDesc {
        Self::Container::deser_type_desc()
    }
    async fn save(&mut self) -> Result<Cid, StoreError> {
        let (container, store) = self.as_container_store();
        container.save(store).await
    }
    async fn save_with_cids(&mut self, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError> {
        let (container, store) = self.as_container_store();
        container.save_with_cids(store, cids_buf).await
    }
    async fn merge(&mut self, other: &Cid) -> Result<(), StoreError> {
        let (container, store) = self.as_container_store();
        container.merge(store, other).await
    }
    async fn diff<'s>(&'s mut self, other: &Cid) -> Result<Self::Container<'s>, StoreError> {
        let (container, store) = self.as_container_store();
        container.diff(store, other).await
    }
}
#[async_trait]
impl<'s, T, S> ContainerWithStore for WithStore<'s, T, S>
where
    T: ContainerV4<'s, S> + 's,
    S: ContentStore,
{
    type Store = S;
    type Container<'s> = T;
    fn deser_type_desc() -> ValueDesc {
        T::deser_type_desc()
    }
    async fn save(&mut self) -> Result<Cid, StoreError> {
        self.container.save(&self.store).await
    }
    async fn save_with_cids(&mut self, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError> {
        self.container.save_with_cids(&self.store, cids_buf).await
    }
    async fn merge(&mut self, other: &Cid) -> Result<(), StoreError> {
        self.container.merge(&self.store, other).await
    }
    async fn diff<'s>(&'s mut self, other: &Cid) -> Result<Self::Container<'s>, StoreError> {
        self.container.diff(&self.store, other).await
    }
}
impl<'s, T, S> TypeDescription for WithStore<'s, T, S>
where
    T: TypeDescription,
{
    fn type_desc() -> ValueDesc {
        T::type_desc()
    }
}
