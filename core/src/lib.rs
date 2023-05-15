use anyhow::anyhow;
use fixity_store::{
    container::{ContainerV4, DefaultContainer, PersistContainer},
    contentid::Cid,
    meta_store::MetaStoreError,
    replicaid::Rid,
    stores::memory::Memory,
    ContentStore, MetaStore,
};
use fixity_structs::replicalog::ReplicaLog;
use std::{
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
    S: ContentStore,
    M: MetaStore,
{
    pub fn new(meta: Arc<M>, store: Arc<S>) -> Self {
        Self { meta, store }
    }
    pub async fn open<T>(&self, repo: &str, replica_id: Rid) -> Result<RepoReplica<M, S, T>, Error>
    where
        T: ContainerV4<S>,
    {
        // TODO: check stored repo type.
        RepoReplica::<M, S, T>::new_open(
            Arc::clone(&self.meta),
            Arc::clone(&self.store),
            repo,
            replica_id,
        )
        .await
    }
}
impl Fixity<Memory<Cid>, Memory<Cid>> {
    /// Construct a new, **in memory only** instance
    pub fn memory() -> Fixity<Memory<Cid>, Memory<Cid>> {
        Fixity {
            meta: Arc::new(Memory::<Cid>::default()),
            store: Arc::new(Memory::<Cid>::default()),
        }
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
pub struct RepoReplica<M, S, T> {
    meta: Arc<M>,
    store: Arc<S>,
    repo: String,
    replica_id: Rid,
    /// Whether or not mutatable access has been granted for the value.
    ///
    /// If `true`, we can use the Head we have stored - if any.
    //
    // NIT: Not sure if needed in the latest iteration. ReplicaLog overlaps
    // in this functionality, we could perhaps defer to it. Though ReplicaLog is just tracking
    // commits, so maybe not enough because mutable access could have been granted for T without
    // ReplicaLog being affected.
    //
    // ReplicaLog perhaps should not bother to track clean then.. /shrug
    clean: bool,
    log: ReplicaLog<S>,
    /// A container or value,
    container: T,
}
impl<M, S, T> RepoReplica<M, S, T>
// NIT: Breakup methods by where clauses. Eg tip() doesn't need ..
// anything.
where
    S: ContentStore,
    M: MetaStore,
    T: ContainerV4<S>,
{
    pub async fn new_open(
        meta: Arc<M>,
        store: Arc<S>,
        repo: &str,
        rid: Rid,
    ) -> Result<Self, Error> {
        let log = match meta.head("local", &rid).await {
            Ok(log_tip) => ReplicaLog::open(&store, &log_tip).await.unwrap(),
            Err(MetaStoreError::NotFound) => ReplicaLog::default_container(&store),
            Err(err) => return Err(Error::Other(anyhow!(err))),
        };
        let (container, new) = match log.repo_tip(repo) {
            Some(tip) => (T::open(&store, &tip).await.unwrap(), false),
            None => (T::default_container(&store), true),
        };
        Ok(Self {
            meta,
            store,
            repo: repo.to_string(),
            replica_id: rid,
            // If the container is new, we start in a modified state.
            // This allows us to commit a zero value, which may make sense depending on some
            // container types.
            //
            // Regardless, starting unclear allows the container to handle zero value writing.
            clean: !new,
            log,
            container,
        })
    }
    /// Return the tip of the associated `Repo`'s `T`.
    pub fn tip(&self) -> Option<Cid> {
        self.log.repo_tip(&self.repo)
    }
    // TODO: Add a method to save_with_inner_cid or something, to allow for reporting
    // both the log_head and inner_head.
    pub async fn commit(&mut self) -> Result<Cid, Error> {
        if self.clean {
            if let Some(tip) = self.tip() {
                return Ok(tip);
            }
        }
        let container_tip = self.container.save(&self.store).await.unwrap();
        self.log.set_repo_tip(&self.repo, container_tip);
        let log_tip = self.log.save(&self.store).await.unwrap();
        self.meta
            .set_head("local", &self.replica_id, log_tip)
            .await
            .unwrap();
        self.clean = true;
        Ok(container_tip)
    }
}
impl<M, S, T> RepoReplica<M, S, T> {}
impl<M, S, T> Deref for RepoReplica<M, S, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}
impl<M, S, T> DerefMut for RepoReplica<M, S, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    #[tokio::test]
    async fn basic_mutation() {
        use fixity_store::replicaid::Rid;
        let rid = Rid::default();
        let mut repo_a = Fixity::memory().open::<String>("foo", rid).await.unwrap();
        // let t = repo_a.deref_mut().await.unwrap();
        // *t = String::from("value");
        // let head_a = repo_a.commit().await.unwrap();
        // dbg!(head_a);
        // {
        //     let mut branch = fixi.branch::<String>("foo", "main", rid).await.unwrap();
        //     let t = branch.inner().await.unwrap();
        //     assert_eq!("value", t);
        // }
        // let mut repo_b = fixi.branch::<String>("bar", "main", rid).await.unwrap();
        // let t = repo_b.inner_mut().await.unwrap();
        // assert_eq!("", t);
        // *t = String::from("value");
        // let head_b = repo_b.commit().await.unwrap();
        // assert_eq!(head_a, head_b);
    }
    /*
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
    */
}
