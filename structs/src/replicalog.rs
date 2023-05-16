use std::{
    any::TypeId,
    collections::{btree_map, BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use fixity_store::{
    container::{
        ContainerDescription, DefaultContainer, DescribeContainer, PersistContainer,
        ReconcileContainer,
    },
    content_store::ContentStore,
    contentid::Cid,
    deser_store::deser_store_v4::DeserExt,
    replicaid::Rid,
    store::StoreError,
};

const DEFAULT_BRANCH: &str = "main";

/// An append only log of all actions for an individual Replica on a Repo. The HEAD of a repo for a
/// Replica. non-CRDT.
#[derive(Debug)]
pub struct ReplicaLog<S> {
    clean: bool,
    // NIT: Not sure if conceptually i want this to be the tip or the head.
    // The difference being, do we allow a user to checkout an arbitrary point making this the head
    // of that point.
    //
    // My gut is no, this is always the tip and for head tracking/exploration, we make it part of
    // the log itself. Otherwise some higher level primitive, like MutStore, is keeping state
    // of two values, the tip and the head. Seems better to have the primitive of only ever
    // needing to track one thing, this value.
    tip_cid: Option<Cid>,
    tip: LogEntry,
    _store: Arc<S>,
}
impl<S> ReplicaLog<S>
where
    S: ContentStore,
{
    pub fn repo_tip(&self, repo_name: &str) -> Option<Cid> {
        self.tip
            .repos
            .repos
            .get(repo_name)
            .map(|repo| repo.branch_tip)
            .clone()
    }
    pub fn set_repo_tip(&mut self, repo: &str, cid: Cid) {
        let modified = match self.tip.repos.repos.entry(repo.to_string()) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(Repo {
                    branch_tip: cid,
                    branches: None,
                });
                true
            },
            btree_map::Entry::Occupied(mut entry) => {
                let repo = entry.get_mut();
                let modified = repo.branch_tip != cid;
                repo.branch_tip = cid;
                modified
            },
        };
        self.clean = self.clean | modified;
    }
}
impl<S> DescribeContainer for ReplicaLog<S> {
    fn description() -> ContainerDescription {
        ContainerDescription {
            name: "ReplicaLog",
            params: Default::default(),
        }
    }
}
// // TODO: Placeholder for signature chain. Need to mock up
// // replica sig and identity sig.
// #[cfg_attr(
//     feature = "rkyv",
//     derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
// )]
// #[derive(Debug)]
// pub struct SignedLogEntry<Rid, Cid> {
//     pub entry: LogEntry<Rid,Cid>,
//     pub replica_sig: (),
// }
// TODO: Make this into an enum. A bit annoying perhaps, but correct, and that's nice.
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug, Default)]
pub struct LogEntry {
    pub previous: Option<Cid>,
    /// [`Defaults`] pointer.
    pub defaults: Option<Cid>,
    /// Embedded [`Repos`] where each [`Repo`] tracks the tip of the active branch.
    pub repos: Repos,
    // /// An [`Identity`] pointer for this Replica.
    // TODO: Move to a ptr, as this data doesn't need to be stored in with active data.
    // pub identity: Option<Cid>,
    pub identity: Option<Identity>,
}
/// Default state of selected repo/branch for stateless use cases, like CLI or app warmups.
///
/// Stateful apps may choose to not use this, so this is only a recommendation.
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug)]
pub struct Defaults {
    /// The name of the active/default repo, for use as the key in [`Repos::repos`].
    pub repo: String,
    /// A default branch per repo.
    pub branches: BTreeMap<String, String>,
}
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug, Default)]
pub struct Repos {
    pub repos: BTreeMap<String, Repo>,
}
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug)]
pub struct Repo {
    // TODO: add Repo type?
    /// The content id of the active branch. The type of this value will be as described by the
    /// Repo.
    ///
    /// Corresponds to the value in [`Defaults::branches`] for the given repo name, as identified
    /// by the key in [`Repos::repos`].
    pub branch_tip: Cid,
    /// A map of `BranchName: HEAD`s to track the various branches that this Replica tracks for the
    /// given repo.
    pub branches: Option<Cid>,
}
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug)]
pub struct Branches {
    /// A map of `BranchName: HEAD`s to track the various branches that this Replica tracks.
    // TODO: Move to a sub container, as this data doesn't need to be stored in with active data.
    // pub branches: Option<Cid>,
    pub branches: BTreeMap<String, Cid>,
}
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug)]
pub struct Identity {
    pub claimed_replicas: BTreeSet<Rid>,
    // pub metadata: CrdtMap<String, Value>
}
impl<S> DefaultContainer<S> for ReplicaLog<S> {
    fn default_container(store: &Arc<S>) -> ReplicaLog<S> {
        ReplicaLog {
            clean: true,
            tip_cid: None,
            tip: Default::default(),
            _store: Arc::clone(store),
        }
    }
}
#[async_trait]
impl<S> PersistContainer<S> for ReplicaLog<S>
where
    S: ContentStore,
{
    async fn open(store: &Arc<S>, cid: &Cid) -> Result<ReplicaLog<S>, StoreError> {
        let tip_cid = Some(cid.clone());
        let tip = store.get_owned_unchecked::<LogEntry>(cid).await?;
        Ok(ReplicaLog {
            clean: true,
            tip_cid,
            tip,
            _store: Arc::clone(store),
        })
    }
    async fn save(&mut self, store: &Arc<S>) -> Result<Cid, StoreError> {
        let previous = match (self.clean, self.tip_cid.take()) {
            // Data is clean, and there is a previous cid, so we can return that and not bother
            // writing an unchanged data structure.
            (true, Some(tip)) => return Ok(tip),
            // Data is clean, but there is no previous. Ie the data is default. Writing default
            // values to the store feels odd, but it's either that or handle it through
            // errors - and a default value can only be written once for any given
            // schema, so this seems a sane behavior.
            (true, None) => None,
            // Data is dirty, write it.
            (false, tip) => tip,
        };
        let entry = &mut self.tip;
        entry.previous = previous;
        // TODO: standardized error, not initialized or something?
        let tip_cid = store.put(&*entry).await?;
        self.tip_cid = Some(tip_cid.clone());
        self.clean = true;
        Ok(tip_cid)
    }
    async fn save_with_cids(
        &mut self,
        store: &Arc<S>,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        let previous = match (self.clean, self.tip_cid.take()) {
            (true, Some(tip)) => {
                cids_buf.push(tip);
                return Ok(());
            },
            (true, None) => None,
            (false, tip) => tip,
        };
        let entry = &mut self.tip;
        entry.previous = previous;
        let entry = &mut self.tip;
        // TODO: standardized error, not initialized or something?
        store.put_with_cids(entry, cids_buf).await?;
        // TODO: add standardized error for cid missing from buf, store did not write to cid buf
        let tip_cid = cids_buf.last().cloned().unwrap();
        self.tip_cid = Some(tip_cid.clone());
        self.clean = true;
        Ok(())
    }
}
#[async_trait]
impl<S> ReconcileContainer<S> for ReplicaLog<S>
where
    S: ContentStore,
{
    async fn merge(&mut self, _store: &Arc<S>, _other: &Cid) -> Result<(), StoreError> {
        Err(StoreError::UnmergableType)
    }
    async fn diff(&mut self, _store: &Arc<S>, _other: &Cid) -> Result<Self, StoreError> {
        Err(StoreError::UndiffableType)
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    use fixity_store::stores::memory::Memory;

    #[tokio::test]
    async fn poc() {
        let store = Arc::new(Memory::default());
        let mut rl = ReplicaLog::default_container(&store);
        rl.set_repo_tip("foo", 1.into());
        dbg!(&rl);
        let cid = rl.save(&store).await.unwrap();
        dbg!(cid, &rl);
        rl.set_repo_tip("foo", 2.into());
        let cid = rl.save(&store).await.unwrap();
        dbg!(cid, &rl);
    }
}
