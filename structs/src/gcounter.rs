use std::collections::BTreeMap;

// TODO: replace with some form of centralized Id type. Likely just
// rand bytes, maybe u64?
type ReplicaId = u64;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug, Default)]
pub struct GCounter(
    // NIT: Replace with ProllyTree.
    // NIT: Make generic to support multiple sizes?
    BTreeMap<ReplicaId, u32>,
);
impl GCounter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn inc(&mut self, rid: ReplicaId) {
        let value = self.0.entry(rid).or_default();
        *value += 1;
    }
    pub fn value(&self) -> u32 {
        self.0.iter().map(|(_, i)| i).sum()
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, rstest::*};
    #[test]
    fn poc() {
        let mut a = GCounter::default();
        a.inc(0);
        assert_eq!(a.value(), 1);
        a.inc(1);
        a.inc(0);
        assert_eq!(a.value(), 3);
    }
}
