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
pub trait ContainerStoreExt {
    fn new_container<T: ContainerV4>(&self) -> WithStore<T, &Self>;
    async fn open<T: ContainerV4>(&self, cid: &Cid) -> Result<WithStore<T, &Self>, StoreError>;
}
#[async_trait]
impl<S> ContainerStoreExt for S
where
    S: ContentStore<Cid>,
{
    fn new_container<T: ContainerV4>(&self) -> WithStore<T, &Self> {
        WithStore {
            container: T::new_container(self),
            store: &self,
        }
    }
    async fn open<T: ContainerV4>(&self, cid: &Cid) -> Result<WithStore<T, &Self>, StoreError> {
        let container = T::open(self, cid).await?;
        Ok(WithStore {
            container,
            store: &self,
        })
    }
}
pub struct WithStore<T, S> {
    container: T,
    store: S,
}
impl<T, S> WithStore<T, S> {
    pub fn into_inner(self) -> T {
        self.container
    }
}
impl<T, S> Deref for WithStore<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}
impl<T, S> DerefMut for WithStore<T, S> {
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
impl<T, S> ContainerWithStore for WithStore<T, S>
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
impl<T, S> TypeDescription for WithStore<T, S>
where
    T: TypeDescription,
{
    fn type_desc() -> ValueDesc {
        T::type_desc()
    }
}
