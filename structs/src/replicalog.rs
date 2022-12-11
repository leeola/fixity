/// An append only log of all actions for an individual Replica on a Repo. The HEAD of a repo for a
/// Replica. non-CRDT.
pub struct ReplicaLog<Rid, Cid> {
    inner: Cid,
    entry: LogEntry<Rid, Cid>,
}
pub struct LogEntry<Rid, Cid> {
    pub previous_entry: Option<Cid>,
    pub type_: LogType<Rid, Cid>,
    // TODO: Placeholder for signature chain. Need to mock up
    // replica sig and identity sig.
    pub _replica_sig: (),
}
pub enum LogType<Rid, Cid> {
    Init { replica_id: Rid },
    // /// A claim that this replica is the same as the other specified replica for
    // /// the given repo.
    //
    // NIT: Maybe metadata should store a map of replica to repo? Depends if
    // we want the same replica being used in multiple repos.
    //
    // /// This claim is only valid if both replicas claim each other.
    // IdentityClaim { repo: String, replica_id: Rid },
    // /// Shared metadata between all
    // IdentityMetadata(CrdtMap<String, Value>)
    // ActiveBranch,
    Commit(CommitLog<Cid>),
}
pub struct CommitLog<Cid> {
    previous: Option<Cid>,
}
