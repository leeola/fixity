/// An append only log of all actions for an individual Replica on a Repo. The HEAD of a repo for a
/// Replica. non-CRDT.
pub struct ReplicaLog<Rid, Cid> {
    inner: Cid,
    entry: LogEntry<Rid, Cid>,
}
pub struct LogEntry<Rid, Cid> {
    pub previous_entry: Option<Cid>,
    pub type_: EntryType<Rid, Cid>,
    // TODO: Placeholder for signature chain. Need to mock up
    // replica sig and identity sig.
    pub _replica_sig: (),
}
pub enum EntryType<Rid, Cid> {
    Init { replica_id: Rid },
    // /// A claim that this replica is the same as the other specified replica.
    // ///
    // /// This claim is only valid if both replicas claim each other.
    // IdentityClaim {
    //     replica_id: Rid,
    //     /// The HEAD at the time of claim.
    //     replica_log_head: Cid,
    //     /// Merged Metadata with the newly claimed replica identity.
    //     identity_metadata: Cid,
    // },
    // /// Mutations against the metadata between all replicas which this replica claims
    // /// to be.
    // IdentityMetadataMutation(CrdtMap<String, Value> CID)
    // ActiveBranch,
    Commit(CommitLog<Cid>),
}
pub struct CommitLog<Cid> {
    previous: Option<Cid>,
}
