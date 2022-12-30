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
    log_head: Option<S::Cid>,
    /// A container or value,
    // value: T,
    log: AppendLog<S::Cid, T, S::Deser>,
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
        let (appendlog, head) = match meta.head("local", repo, branch, &rid).await {
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
            log_head: head,
            log: appendlog,
        })
    }
    pub async fn head(&self) -> Option<S::Cid> {
        self.log_head.clone()
    }
    pub async fn commit(&mut self) -> Result<S::Cid, Error>
    where
        S: Store<Deser = Rkyv>,
        T: Deserialize<Rkyv>,
        S::Cid: Deserialize<Rkyv>,
    {
        if self.clean {
            if let Some(head) = self.log_head.clone() {
                return Ok(head);
            }
            // if clean, but no head - this is an initial value, eg T::init(),
            // We prevent writing init data by default, as that's likely useless
            // state. Adding a future `commit_init` would bypass this.
            return Err(Error::CommitInitValue);
        }
        // TODO: Add a method to save_with_inner_cid or something, to allow for reporting
        // both the log_head and inner_head.
        let log_head = self.log.save(&*self.store).await.unwrap();
        if let Some(current_log_head) = self.log_head.clone() {
            if log_head == current_log_head {
                // TODO: save the data cid alongside the log_head, methinks. Avoid this.
                let data_cid = self.log.inner_cid(&*self.store).await.unwrap();
                let data_cid = data_cid.unwrap();
                return Ok(data_cid);
            }
        }
        self.meta
            .set_head(
                "local",
                &self.repo,
                &self.branch,
                &self.replica_id,
                log_head.clone(),
            )
            .await
            .unwrap();
        self.log_head = Some(log_head.clone());
        self.clean = true;
        let data_cid = self.log.inner_cid(&*self.store).await.unwrap();
        let data_cid = data_cid.unwrap();
        Ok(data_cid)
    }
}
// NIT: Rkyv specific impls. Required current because AppendLog only impls for Rkyv.
//
// AppendLog should probably make it fully generic for the health of the primary APIs,
// since it's such a core type.
impl<M, S, T> RepoReplica<M, S, T>
where
    S: Store<Deser = Rkyv>,
    M: Meta<S::Cid>,
    S::Cid: Deserialize<S::Deser>,
    T: Deserialize<S::Deser>,
    AppendNode<S::Cid, S::Cid>: Deserialize<S::Deser>,
{
    // NIT: requiring mut on a inner() method feels bad. AppendLog should be changed
    // so it's not so abusive. Even if for less perf, perhaps.
    pub async fn inner(&mut self) -> Result<&T, Error> {
        let t = self.log.inner(&*self.store).await.unwrap();
        Ok(t)
    }
    pub async fn inner_mut(&mut self) -> Result<&mut T, Error> {
        self.clean = false;
        let t = self.log.inner_mut(&*self.store).await.unwrap();
        Ok(t)
    }
    pub async fn commit_value<U: Into<T>>(&mut self, value: U) -> Result<S::Cid, Error>
    where
        for<'s> T: Container<'s, S>,
        for<'s> AppendLog<S::Cid, T, S::Deser>: Container<'s, S>,
        AppendNode<S::Cid, S::Cid>: Serialize<S::Deser>,
    {
        let self_inner = self.inner_mut().await?;
        *self_inner = value.into();
        self.commit().await
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    #[tokio::test]
    async fn basic_mutation() {
        use fixity_store::replicaid::Rid;
        let rid = Rid::<8>::default();
        let fixi = Fixity::memory();
        let mut repo_a = fixi.branch::<String>("foo", "main", rid).await.unwrap();
        let t = repo_a.inner_mut().await.unwrap();
        *t = String::from("value");
        let head_a = repo_a.commit().await.unwrap();
        dbg!(head_a);
        {
            let mut branch = fixi.branch::<String>("foo", "main", rid).await.unwrap();
            let t = branch.inner().await.unwrap();
            assert_eq!("value", t);
        }
        let mut repo_b = fixi.branch::<String>("bar", "main", rid).await.unwrap();
        let t = repo_b.inner_mut().await.unwrap();
        assert_eq!("", t);
        *t = String::from("value");
        let head_b = repo_b.commit().await.unwrap();
        assert_eq!(head_a, head_b);
    }
    #[tokio::test]
    async fn reports_inner_cid() {
        use fixity_store::replicaid::Rid;
        let rid = Rid::<8>::default();
        let fixi = Fixity::memory();
        let mut repo = fixi.branch::<String>("foo", "main", rid).await.unwrap();
        let head_foo_a = repo.commit_value("foo").await.unwrap();
        let head_bar = repo.commit_value("bar").await.unwrap();
        assert_ne!(head_foo_a, head_bar);
        let head_foo_b = repo.commit_value("foo").await.unwrap();
        assert_eq!(head_foo_a, head_foo_b);
    }
}
