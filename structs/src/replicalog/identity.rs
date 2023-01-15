use std::collections::BTreeSet;

pub struct Identity<Rid> {
    pub claimed_replicas: BTreeSet<Rid>,
    // pub metadata: CrdtMap<String, Value>
}
