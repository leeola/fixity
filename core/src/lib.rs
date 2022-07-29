use async_trait::async_trait;
use fixity_store::{Meta, Store};
use std::{marker::PhantomData, ops::Deref, sync::Arc};

pub struct Fixity<Meta, Store> {
    meta: Arc<Meta>,
    store: Arc<Store>,
}
impl<M, S> Fixity<M, S> {
    pub fn new(meta: Arc<M>, store: Arc<S>) -> Self {
        Self { meta, store }
    }
    pub async fn open<T>(&self, repo: &str) -> Repo<M, S, T> {
        todo!()
    }
}
pub struct Repo<Meta, Store, T> {
    meta: Arc<Meta>,
    store: Arc<Store>,
    _t: PhantomData<T>,
}
impl<M, S, T> Repo<M, S, T> {
    pub async fn open(meta: Arc<M>, store: Arc<S>, repo: &str) -> Self {
        todo!()
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
pub struct RepoReplica<Meta, Store, T, Rid> {
    meta: Arc<Meta>,
    store: Arc<Store>,
    t: PhantomData<T>,
    replica_id: Rid,
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
#[tokio::test]
async fn wip() {
    use fixity_store::{cid::Hasher, deser::Rkyv, store::Memory};
    let fixi = {
        let mem = Arc::new(Memory::<Rkyv, Hasher>::new());
        let repo = Fixity::new(Arc::clone(&mem), mem)
            .open::<String>("foo")
            .await;
    };
}
