use std::collections::{BTreeMap, BTreeSet};

/// An append only log of all actions for an individual Replica on a Repo. The HEAD of a repo for a
/// Replica. non-CRDT.
#[derive(Debug, Default)]
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
#[derive(Debug)]
pub struct Identity<Rid> {
    pub claimed_replicas: BTreeSet<Rid>,
    // pub metadata: CrdtMap<String, Value>
}
#[cfg(test)]
pub mod test {
    use fixity_store::stores::memory::Memory;

    use super::*;
    #[tokio::test]
    async fn poc() {
        let store = Memory::test();
        let r = ReplicaLog::default();
        // let b_cid = b.save(&store).await.unwrap();
        // a.merge(&store, &b_cid).await.unwrap();
        // assert_eq!(a.value(), 3);
        // b.inc(1);
        // let b_cid = b.save(&store).await.unwrap();
        // a.merge(&store, &b_cid).await.unwrap();
        // assert_eq!(a.value(), 4);
    }
}
