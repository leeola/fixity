use {
    async_trait::async_trait,
    fixity_store::prelude::{Content, ContentHasher, Error, Store},
    std::collections::{btree_map::Entry, BTreeMap},
};

// TODO: replace with some form of centralized Id type. Likely just
// rand bytes, maybe u64?
pub type ReplicaId = u64;
type GCounterInt = u32;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug))]
pub struct GCounter(
    // NIT: Optionally use ProllyTree for large concurrent uses.
    // NIT: Make generic to support multiple sizes?
    BTreeMap<ReplicaId, GCounterInt>,
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
    pub fn contains_rid(&self, rid: &ReplicaId) -> bool {
        self.0.contains_key(rid)
    }
    pub fn get(&self, rid: &ReplicaId) -> Option<&GCounterInt> {
        self.0.get(rid)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&ReplicaId, &GCounterInt)> {
        self.0.iter()
    }
    pub fn merge(&mut self, other: &Self) {
        for (&rid, &other_i) in other.0.iter() {
            match self.0.entry(rid) {
                Entry::Occupied(mut entry) => {
                    let self_i = entry.get_mut();
                    *self_i = (*self_i).max(other_i);
                },
                Entry::Vacant(entry) => {
                    entry.insert(other_i);
                },
            }
        }
    }
}
#[async_trait]
impl<S, H> Content<Self, S, H> for GCounter
where
    S: Store<Self, H> + Sync,
    H: ContentHasher,
{
    async fn get(store: &S, cid: &H::Cid) -> Result<S::Repr, Error> {
        todo!()
    }
    async fn put_and_head(&self, store: &S) -> Result<H::Cid, Error> {
        todo!()
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        fixity_store::store::{json_store::JsonStore, rkyv_store::RkyvStore, Repr, Store},
        rstest::*,
        std::fmt::Debug,
    };
    #[test]
    fn poc() {
        let mut a = GCounter::default();
        a.inc(0);
        assert_eq!(a.value(), 1);
        a.inc(1);
        a.inc(0);
        assert_eq!(a.value(), 3);
        let mut b = GCounter::default();
        b.inc(1);
        b.inc(1);
        b.merge(&a);
        assert_eq!(b.value(), 4);
    }
    #[rstest]
    #[case::json(JsonStore::memory())]
    #[case::rkyv(RkyvStore::memory())]
    #[tokio::test]
    async fn content_io<S>(#[case] store: S)
    where
        S: Store<GCounter> + Sync,
        <<S as Store<GCounter>>::Repr as Repr>::Borrow: Debug + PartialEq<GCounter>,
    {
        let mut counter = GCounter::default();
        counter.inc(0);
        let cid = counter.put_and_head(&store).await.unwrap();
        // let repr = Content::<GCounter, S, fixity_store::cid::Hasher>::get(&store, &cid)
        let repr = <GCounter as Content<_, _, _>>::get(&store, &cid)
            .await
            .unwrap();
        assert_eq!(repr.repr_borrow().unwrap(), &counter);
    }
}
