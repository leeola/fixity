use async_trait::async_trait;
use fixity_store::{
    container::Container,
    contentid::{Hasher, CID_LENGTH},
    deser::Rkyv,
    storage,
    store::{self, StoreImpl},
    Meta, Store,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};
pub type Error = ();
pub struct Fixity<Meta, Store> {
    meta: Arc<Meta>,
    store: Arc<Store>,
}
impl<M, S> Fixity<M, S>
where
    S: Store,
    M: Meta<S::Cid>,
{
    pub fn new(meta: Arc<M>, store: Arc<S>) -> Self {
        Self { meta, store }
    }
    pub async fn open<T: Container<S>>(&self, repo: &str) -> Result<Repo<M, S, T>, Error> {
        // TODO: check stored repo type. Meta doesn't store
        // repo signature yet.
        Repo::<M, S, T>::open(Arc::clone(&self.meta), Arc::clone(&self.store), repo).await
    }
}
// Some type aliases for simplicity.
type MemStorage = storage::Memory;
type MemStore = store::StoreImpl<Arc<MemStorage>, Rkyv, Hasher>;
type MemFixity = Fixity<MemStorage, MemStore>;
impl MemFixity {
    pub fn memory() -> MemFixity {
        let storage = Arc::new(storage::Memory::default());
        let store = Arc::new(StoreImpl::new(Arc::clone(&storage)));
        MemFixity::new(storage, store)
    }
}
pub struct Repo<Meta, Store, T> {
    meta: Arc<Meta>,
    store: Arc<Store>,
    repo: Box<str>,
    _phantom_t: PhantomData<T>,
}
impl<M, S, T> Repo<M, S, T>
where
    S: Store,
    M: Meta<S::Cid>,
    T: Container<S>,
{
    pub async fn open(meta: Arc<M>, store: Arc<S>, repo: &str) -> Result<Self, Error> {
        Ok(Repo {
            repo: Box::from(repo),
            meta,
            store,
            _phantom_t: PhantomData,
        })
    }
    pub async fn branch(
        &self,
        branch: &str,
        replica: M::Rid,
    ) -> Result<RepoReplica<M, S, T>, Error> {
        RepoReplica::open(
            Arc::clone(&self.meta),
            Arc::clone(&self.store),
            &self.repo,
            branch,
            replica,
        )
        .await
    }
}
// TODO: figure out how the Containers get access to meta/store/HEAD tracking.
// A: Maybe none needed? Repo creates the instance of T from a `Container::new(head)`
// and due to it being a replica, everything is safe after.
// Q: How does the Container update the head?
// Q2: Is there a difference between root interface and child content interfaces?
//     The root needs to update a pointer, the rest just write.
// A: Try wrapping the inner `T` and `Defer/Mut` into it. Then `Replica::commit()` will
// write it, and then update the pointer.
// That also lets us track mut and do nothing if it was never mutated.
pub struct RepoReplica<M, S, T>
where
    S: Store,
    M: Meta<S::Cid>,
{
    meta: Arc<M>,
    store: Arc<S>,
    repo: Box<str>,
    branch: Box<str>,
    replica_id: M::Rid,
    value: T,
}
impl<M, S, T> RepoReplica<M, S, T>
where
    S: Store,
    M: Meta<S::Cid>,
{
    pub async fn open(
        meta: Arc<M>,
        store: Arc<S>,
        repo: &str,
        branch: &str,
        replica: M::Rid,
    ) -> Result<Self, Error> {
        // let head = meta.head().await?;
        todo!()
    }
}
impl<M, S, T> Deref for RepoReplica<M, S, T>
where
    S: Store,
    M: Meta<S::Cid>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<M, S, T> DerefMut for RepoReplica<M, S, T>
where
    S: Store,
    M: Meta<S::Cid>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
pub mod api_drafting {
    use async_trait::async_trait;
    use std::collections::HashSet;
    #[async_trait]
    pub trait WriteSer<Cid> {
        async fn write_serialize(&self, store: ()) -> Result<Cid, ()>;
    }
    pub struct FooContainer<T> {
        foo: Foo<T>,
    }
    trait ContentContainer {
        type DeserType; // :Deser bound,
        fn write(&mut self, store: ()) -> ();
    }
    // IDEA: maybe track loaded ptrs with hierarchy so that a centralized location
    // can write them in reverse order, efficiently.
    pub enum Ptr<T> {
        Ptr {
            cid: (),
        },
        Ref {
            cid: (),
            value: T,
            // children: Vec<Ptr<U>>, // !?
        },
        Mut {
            previous_cid: (),
            value: T,
        },
    }
    pub struct Foo<T> {
        items: Option<Ptr<T>>,
    }
}
pub mod api_drafting_2 {
    pub struct PtrOwner<T, V> {
        // inner container thing, userland type.
        inner: T,
        // registries, but inner can prob return these
        // via Trait?
        registries: V, // Can be (V1,V2,V3,etc)
    }
    pub struct PtrRegistry<V>(V);
}
pub mod api_drafting_3 {
    use std::{collections::HashMap, sync::Arc};

    // NIT: Is there something cheaper than Arc? Since
    // i don't care about using the Rc portion of Arc.
    pub struct Ptr<T>(Arc<PtrInner<T>>);
    enum PtrInner<T> {
        Ptr { cid: () },
        Ref { cid: (), value: T },
        Mut { value: T },
    }
    pub struct PtrRegistry<Cid, Container, T> {
        container: Container,
        weak_ptrs: HashMap<Cid, Ptr<T>>,
    }
}
#[cfg(test)]
#[tokio::test]
async fn wip() {
    use fixity_store::{contentid::Hasher, deser::Rkyv, store::Memory};
    let repo = Fixity::memory().open::<String>("foo").await.unwrap();
}