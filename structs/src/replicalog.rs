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
    pub previous_entry: Option<Cid>,
    /// A map of `BranchName: HEAD`s to track the various branches that this Replica tracks.
    pub branches: Option<Branches<Cid>>,
    // /// An [`Identity`] pointer for this Replica.
    // TODO: Move to a sub container, as this data doesn't need to be stored in with active data.
    // pub identity: Option<Cid>,
    pub identity: Option<Identity<Rid>>,
    // // TODO: Placeholder for signature chain. Need to mock up
    // // replica sig and identity sig.
    // pub _replica_sig: (),
}
impl<Rid, Cid> ReplicaLog<Rid, Cid> {
    pub fn new() -> Self {
        Self::default()
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
            values: vec![
                ValueDesc::of::<Option<Cid>>(),
                ValueDesc::of::<Option<Branches<Cid>>>(),
                ValueDesc::of::<Option<Identity<Rid>>>(),
            ],
        }
    }
}
impl<Rid, Cid> Default for ReplicaLog<Rid, Cid> {
    fn default() -> Self {
        Self {
            previous_entry: Default::default(),
            branches: Default::default(),
            identity: Default::default(),
        }
    }
}
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
    Rid: NewReplicaId,
    Cid: NewContentId,
    Self: Serialize<Rkyv> + Deserialize<Rkyv>,
{
    fn deser_type_desc() -> ValueDesc {
        Self::type_desc()
    }
    fn new_container<S: DeserStore<Rkyv, Cid>>(_: &S) -> Self {
        Self::default()
    }
    async fn open<S: DeserStore<Rkyv, Cid>>(store: &S, cid: &Cid) -> Result<Self, StoreError> {
        let repr = store.get::<Self>(cid).await?;
        let self_ = repr.repr_to_owned()?;
        Ok(self_)
    }
    async fn save<S: DeserStore<Rkyv, Cid>>(&mut self, store: &S) -> Result<Cid, StoreError> {
        store.put(self).await
    }
    async fn save_with_cids<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        store.put_with_cids(self, cids_buf).await
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
        let store = Memory::test();
        let rl = ReplicaLog::default();
        let b_cid = rl.save(&store).await.unwrap();
        // a.merge(&store, &b_cid).await.unwrap();
        // assert_eq!(a.value(), 3);
        // b.inc(1);
        // let b_cid = b.save(&store).await.unwrap();
        // a.merge(&store, &b_cid).await.unwrap();
        // assert_eq!(a.value(), 4);
    }
}
