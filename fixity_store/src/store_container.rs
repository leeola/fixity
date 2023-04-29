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
pub trait StoreContainerExt {
    fn new_container<T: ContainerV4>(&self) -> WithStore<&Self, T>;
    async fn open<T: ContainerV4>(&self, cid: &Cid) -> WithStore<&Self, T>;
}
pub struct WithStore<S, T> {
    container: T,
    store: S,
}
impl<S, T> WithStore<S, T> {
    pub fn into_inner(self) -> T {
        self.container
    }
}
impl<S, T> Deref for WithStore<S, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}
impl<S, T> DerefMut for WithStore<S, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
/// A trait to wrap a [`Container`] and pass in an associated store to container methods..
#[async_trait]
pub trait ContainerWithStore: Sized + Send + TypeDescription {
    type Container: ContainerV4;
    fn deser_type_desc() -> ValueDesc;
    async fn save(&mut self) -> Result<Cid, StoreError>;
    async fn save_with_cids(&mut self, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>;
    async fn merge(&mut self, other: &Cid) -> Result<(), StoreError>;
    async fn diff(&mut self, other: &Cid) -> Result<Self::Container, StoreError>;
}
#[async_trait]
impl<S, T> ContainerWithStore for WithStore<S, T>
where
    T: ContainerV4,
    S: ContentStore<Cid>,
{
    type Container = T;
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
    async fn diff(&mut self, other: &Cid) -> Result<Self::Container, StoreError> {
        self.container.diff(&self.store, other).await
    }
}
impl<S, T> TypeDescription for WithStore<S, T>
where
    T: TypeDescription,
{
    fn type_desc() -> ValueDesc {
        T::type_desc()
    }
}
