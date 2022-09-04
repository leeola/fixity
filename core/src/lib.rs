use anyhow::{anyhow, bail};
use async_trait::async_trait;
use fixity_store::{
    container::Container,
    contentid::{Hasher, CID_LENGTH},
    deser::{Deserialize, Rkyv, Serialize},
    meta::MetaStoreError,
    storage,
    store::{self, StoreImpl},
    Meta, Store,
};
use fixity_structs::appendlog::{AppendLog, AppendNode};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot implicitly commit an initial value")]
    CommitInitValue,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
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
    pub async fn open<T>(&self, repo: &str) -> Result<Repo<M, S, T>, Error>
    where
        S::Cid: Serialize<S::Deser> + Deserialize<S::Deser>,
        for<'s> T: Container<'s, S> + Sync,
    {
        // TODO: check stored repo type. Meta doesn't store
        // repo signature yet.
        Repo::<M, S, T>::new_open(Arc::clone(&self.meta), Arc::clone(&self.store), repo).await
    }
    pub async fn branch<T>(
        &self,
        repo: &str,
        branch: &str,
        replica: M::Rid,
    ) -> Result<RepoReplica<M, S, T>, Error>
    where
        S::Cid: Serialize<S::Deser> + Deserialize<S::Deser>,
        for<'s> T: Container<'s, S> + Sync,
        AppendNode<S::Cid, S::Cid>: Serialize<S::Deser> + Deserialize<S::Deser>,
        S::Deser: 'static,
    {
        self.open::<T>(repo).await?.branch(branch, replica).await
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
    S::Cid: Serialize<S::Deser> + Deserialize<S::Deser>,
    for<'s> T: Container<'s, S> + Sync,
{
    pub async fn new_open(meta: Arc<M>, store: Arc<S>, repo: &str) -> Result<Self, Error> {
        Ok(Repo {
            repo: Box::from(repo),
            meta,
            store,
            _phantom_t: PhantomData,
        })
    }
    pub async fn branch(&self, branch: &str, replica: M::Rid) -> Result<RepoReplica<M, S, T>, Error>
    where
        AppendNode<S::Cid, S::Cid>: Serialize<S::Deser> + Deserialize<S::Deser>,
        S::Deser: 'static,
    {
        RepoReplica::new_open(
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
    /// Whether or not mutatable access has been granted for the value.
    ///
    /// If `true`, we can use the Head we have stored - if any.
    clean: bool,
    /// The cid reported by the `MetaStore`, used to load the most recent
    /// value for this branch replica.
    head: Option<S::Cid>,
    /// A container or value,
    // value: T,
    value: AppendLog<S::Cid, T, S::Deser>,
}
impl<M, S, T> RepoReplica<M, S, T>
where
    S: Store,
    M: Meta<S::Cid>,
    for<'s> T: Container<'s, S>,
    for<'s> AppendLog<S::Cid, T, S::Deser>: Container<'s, S>,
    AppendNode<S::Cid, S::Cid>: Serialize<S::Deser> + Deserialize<S::Deser>,
{
    pub async fn new_open(
        meta: Arc<M>,
        store: Arc<S>,
        repo: &str,
        branch: &str,
        rid: M::Rid,
    ) -> Result<Self, Error> {
        let (value, head) = match meta.head("local", repo, branch, &rid).await {
            Ok(head) => (AppendLog::open(&*store, &head).await.unwrap(), Some(head)),
            Err(MetaStoreError::NotFound) => (AppendLog::new(&*store), None),
            Err(err) => return Err(Error::Other(anyhow!(err))),
        };
        Ok(Self {
            meta,
            store,
            repo: Box::from(repo),
            branch: Box::from(branch),
            replica_id: rid,
            clean: true,
            head,
            value,
        })
    }
    pub async fn commit(&mut self) -> Result<S::Cid, Error> {
        if self.clean {
            if let Some(head) = self.head.clone() {
                return Ok(head);
            }
            // if clean, but no head - this is an initial value, eg T::init(),
            // We prevent writing init data by default, as that's likely useless
            // state. Adding a future `commit_init` would bypass this.
            return Err(Error::CommitInitValue);
        }
        let cid = self.value.save(&*self.store).await.unwrap();
        self.meta
            .set_head(
                "local",
                &*self.repo,
                &*self.branch,
                &self.replica_id,
                cid.clone(),
            )
            .await
            .unwrap();
        self.head = Some(cid.clone());
        self.clean = true;
        Ok(cid)
    }
}
#[cfg(test)]
#[tokio::test]
async fn basic_mutation() {
    use fixity_store::replicaid::Rid;
    let rid = Rid::<8>::default();
    let fixi = Fixity::memory();
    let mut repo_a = fixi.branch::<String>("foo", "main", rid).await.unwrap();
    // let t = repo_a.deref_mut();
    // *t = String::from("value");
    /*
    let head_a = repo_a.commit().await.unwrap();
    dbg!(head_a);
    {
        let repo = fixi.branch::<String>("foo", "main", rid).await.unwrap();
        let t = repo.deref();
        assert_eq!("value", t);
    }
    let mut repo_b = fixi.branch::<String>("bar", "main", rid).await.unwrap();
    let t = repo_b.deref_mut();
    assert_eq!("", t);
    *t = String::from("value");
    let head_b = repo_b.commit().await.unwrap();
    assert_eq!(head_a, head_b);
    */
}
