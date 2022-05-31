use {
    async_trait::async_trait,
    fixity_store::{RemoteStorage, Store},
    std::{marker::PhantomData, ops::Deref, sync::Arc},
};

pub struct Repo<Remote, Store> {
    remote_storage: Arc<Remote>,
    store: Arc<Store>,
}
impl<R, S> Repo<R, S> {
    pub async fn open(remote: Arc<R>, store: Arc<S>, repo: &str) -> Self {
        todo!()
    }
}
