use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
};

use async_trait::async_trait;
use fixity_store::{
    container::ContainerV4,
    content_store::ContentStore,
    contentid::Cid,
    deser_store::deser_store_v4::DeserExt,
    replicaid::Rid,
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
};

/// An append only log of all actions for an individual Replica on a Repo. The HEAD of a repo for a
/// Replica. non-CRDT.
#[derive(Debug)]
pub struct ReplicaLog {
    entry: Option<LogEntry>,
}
impl ReplicaLog {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_commit(&mut self, repo: String, cid: Cid) {
        // let entry = self.entry.get_or_insert_default();
        // let defaults = entry.defaults
        // match self.entry.as_mut() {
        //     Some(entry) => {
        //         entry.branches.content = cid;
        //     },
        //     None => {
        //         self.entry = Some(LogEntry {
        //             previous_entry: None,
        //             branches: Branches {
        //                 active: "main".to_string(),
        //                 content: cid,
        //                 inactive: Default::default(),
        //             },
        //             identity: Default::default(),
        //         })
        //     },
        // }
        todo!()
    }
}
impl Default for ReplicaLog {
    fn default() -> Self {
        Self {
            entry: Default::default(),
        }
    }
}
impl TypeDescription for ReplicaLog {
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "ReplicaLog",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<Rid>(), ValueDesc::of::<Cid>()],
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
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug)]
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
#[derive(Debug)]
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
impl TypeDescription for Branches {
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "Branches",
            type_id: TypeId::of::<Self>(),
            values: vec![
                ValueDesc::of::<String>(),
                ValueDesc::of::<Cid>(),
                ValueDesc::of::<BTreeMap<String, Cid>>(),
            ],
        }
    }
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
impl TypeDescription for Identity {
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "Identity",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<BTreeSet<Rid>>()],
        }
    }
}
#[async_trait]
impl<S> ContainerV4<S> for ReplicaLog
where
    S: ContentStore,
{
    fn deser_type_desc() -> ValueDesc {
        Self::type_desc()
    }
    fn new_container(_: &S) -> Self {
        Self::default()
    }
    async fn open(store: &S, cid: &Cid) -> Result<Self, StoreError> {
        let entry = store.get_owned_unchecked::<LogEntry>(cid).await?;
        Ok(Self { entry: Some(entry) })
    }
    async fn save(&mut self, store: &S) -> Result<Cid, StoreError> {
        // TODO: standardized error, not initialized or something?
        let entry = self.entry.as_mut().unwrap();
        let cid = store.put(&*entry).await?;
        entry.previous = Some(cid.clone());
        Ok(cid)
    }
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        // TODO: standardized error, not initialized or something?
        let entry = self.entry.as_mut().unwrap();
        store.put_with_cids(entry, cids_buf).await?;
        // TODO: add standardized error for cid missing from buf, store did not write to cid buf
        let cid = cids_buf.last().cloned().unwrap();
        entry.previous = Some(cid);
        Ok(())
    }
    async fn merge(&mut self, _store: &S, _other: &Cid) -> Result<(), StoreError> {
        Err(StoreError::UnmergableType)
    }
    async fn diff(&mut self, _store: &S, _other: &Cid) -> Result<Self, StoreError> {
        Err(StoreError::UndiffableType)
    }
}
#[cfg(test)]
pub mod test {
    use fixity_store::stores::memory::Memory;

    use super::*;
    #[tokio::test]
    async fn poc() {
        let store = Memory::default();
        let mut rl = ReplicaLog::default();
        rl.set_commit(1.into());
        dbg!(&rl);
        let cid = rl.save(&store).await.unwrap();
        dbg!(cid, &rl);
        rl.set_commit(2.into());
        let cid = rl.save(&store).await.unwrap();
        dbg!(cid, &rl);
    }
}
