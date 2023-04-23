use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
};

use async_trait::async_trait;
use fixity_store::{
    container::NewContainer,
    contentid::NewContentId,
    deser::{Deserialize, Rkyv, Serialize},
    deser_store::DeserStore,
    replicaid::NewReplicaId,
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
};

/// An append only log of all actions for an individual Replica on a Repo. The HEAD of a repo for a
/// Replica. non-CRDT.
#[derive(Debug)]
pub struct ReplicaLog<Rid, Cid> {
    entry: Option<LogEntry<Rid, Cid>>,
}
impl<Rid, Cid> ReplicaLog<Rid, Cid> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_commit(&mut self, cid: Cid) {
        match self.entry.as_mut() {
            Some(entry) => {
                entry.branches.content = cid;
            },
            None => {
                self.entry = Some(LogEntry {
                    previous_entry: None,
                    branches: Branches {
                        active: "main".to_string(),
                        content: cid,
                        inactive: Default::default(),
                    },
                    identity: Default::default(),
                })
            },
        }
    }
}
impl<Rid, Cid> Default for ReplicaLog<Rid, Cid> {
    fn default() -> Self {
        Self {
            entry: Default::default(),
        }
    }
}
impl<Rid, Cid> TypeDescription for ReplicaLog<Rid, Cid>
where
    Rid: NewReplicaId,
    Cid: NewContentId,
{
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
pub struct LogEntry<Rid, Cid> {
    pub previous_entry: Option<Cid>,
    // TODO: Move under a new `Repo<>` struct.
    /// A map of `BranchName: HEAD`s to track the various branches that this Replica tracks.
    pub branches: Branches<Cid>,
    // /// An [`Identity`] pointer for this Replica.
    // TODO: Move to a sub container, as this data doesn't need to be stored in with active data.
    // pub identity: Option<Cid>,
    pub identity: Option<Identity<Rid>>,
}
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug)]
pub struct Branches<Cid> {
    /// The name of the active branch.
    pub active: String,
    /// The content id of the active branch.
    pub content: Cid,
    /// A map of `BranchName: HEAD`s to track the various branches that this Replica tracks.
    // TODO: Move to a sub container, as this data doesn't need to be stored in with active data.
    // pub branches: Option<Cid>,
    pub inactive: BTreeMap<String, Cid>,
}
impl<Cid> TypeDescription for Branches<Cid>
where
    Cid: NewContentId,
{
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
pub struct Identity<Rid> {
    pub claimed_replicas: BTreeSet<Rid>,
    // pub metadata: CrdtMap<String, Value>
}
impl<Rid> TypeDescription for Identity<Rid>
where
    Rid: NewReplicaId,
{
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "Identity",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<BTreeSet<Rid>>()],
        }
    }
}
#[async_trait]
impl<Rid, Cid> NewContainer<Rkyv, Cid> for ReplicaLog<Rid, Cid>
where
    Rid: NewReplicaId + Serialize<Rkyv> + Deserialize<Rkyv>,
    Cid: NewContentId + Serialize<Rkyv> + Deserialize<Rkyv>,
    LogEntry<Rid, Cid>: Serialize<Rkyv> + Deserialize<Rkyv>,
{
    fn deser_type_desc() -> ValueDesc {
        Self::type_desc()
    }
    fn new_container<S: DeserStore<Rkyv, Cid>>(_: &S) -> Self {
        Self::default()
    }
    async fn open<S: DeserStore<Rkyv, Cid>>(store: &S, cid: &Cid) -> Result<Self, StoreError> {
        let repr = store.get::<LogEntry<Rid, Cid>>(cid).await?;
        let entry = repr.repr_to_owned()?;
        Ok(Self { entry: Some(entry) })
    }
    async fn save<S: DeserStore<Rkyv, Cid>>(&mut self, store: &S) -> Result<Cid, StoreError> {
        // TODO: standardized error, not initialized or something?
        let entry = self.entry.as_mut().unwrap();
        let cid = store.put(&*entry).await?;
        entry.previous_entry = Some(cid.clone());
        Ok(cid)
    }
    async fn save_with_cids<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        // TODO: standardized error, not initialized or something?
        let entry = self.entry.as_mut().unwrap();
        store.put_with_cids(entry, cids_buf).await?;
        // TODO: add standardized error for cid missing from buf, store did not write to cid buf
        let cid = cids_buf.last().cloned().unwrap();
        entry.previous_entry = Some(cid);
        Ok(())
    }
    async fn merge<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        _store: &S,
        _other: &Cid,
    ) -> Result<(), StoreError> {
        Err(StoreError::UnmergableType)
    }
    async fn diff<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        _store: &S,
        _other: &Cid,
    ) -> Result<Self, StoreError> {
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
        let mut rl = ReplicaLog::<i32, _>::default();
        rl.set_commit(1);
        dbg!(&rl);
        let cid = rl.save(&store).await.unwrap();
        dbg!(cid, &rl);
        rl.set_commit(2);
        let cid = rl.save(&store).await.unwrap();
        dbg!(cid, &rl);
    }
}
