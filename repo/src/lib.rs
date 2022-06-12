use {
    async_trait::async_trait,
    fixity_store::{Meta, Store},
    std::{marker::PhantomData, ops::Deref, sync::Arc},
};

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

pub struct RepoReplica<Meta, Store, T, Rid> {
    meta: Arc<Meta>,
    store: Arc<Store>,
    _t: PhantomData<T>,
    replica_id: Rid,
}
