use async_trait::async_trait;
use fixity_store::{
    cid::{Hasher, CID_LENGTH},
    container::Container,
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
type Rid = [u8; 8];
pub struct Fixity<Meta, Store> {
    meta: Arc<Meta>,
    store: Arc<Store>,
}
impl<M, S> Fixity<M, S>
where
    S: Store,
    M: Meta<Rid, S::Cid>,
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
    repo: String,
    meta: Arc<Meta>,
    store: Arc<Store>,
    _phantom_t: PhantomData<T>,
}
impl<M, S, T> Repo<M, S, T>
where
    S: Store,
    T: Container<S>,
{
    pub async fn open(meta: Arc<M>, store: Arc<S>, repo: &str) -> Result<Self, Error> {
        Ok(Repo {
            repo: repo.to_string(),
            meta,
            store,
            _phantom_t: PhantomData,
        })
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
pub struct RepoReplica<Meta, Store, Rid, T> {
    meta: Arc<Meta>,
    store: Arc<Store>,
    replica_id: Rid,
    value: T,
}
impl<M, S, R, T> RepoReplica<M, S, R, T> {
    pub async fn open(meta: Arc<M>, store: Arc<S>, replica: R) -> Self {
        todo!()
    }
}
impl<M, S, R, T> Deref for RepoReplica<M, S, R, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<M, S, R, T> DerefMut for RepoReplica<M, S, R, T> {
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
    use fixity_store::{cid::Hasher, deser::Rkyv, store::Memory};
    let repo = Fixity::memory().open::<String>("foo").await.unwrap();
}
