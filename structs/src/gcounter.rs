use {
    async_trait::async_trait,
    fixity_store::{Content, ContentHasher, Error, Repr, Store},
    std::collections::{btree_map::Entry, BTreeMap},
};

// TODO: replace with some form of centralized Id type. Likely just
// rand bytes, maybe u64?
pub type ReplicaId = u64;
type GCounterInt = u32;

pub enum CidPtr<Cid, T, R> {
    Cid(Cid),
    Owned(T),
    Repr { cid: Cid, repr: R },
}

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
impl<S, H> Content<S, H> for GCounter
where
    S: Store<Self, H> + Sync,
    H: ContentHasher,
{
    async fn load(store: &S, cid: &H::Cid) -> Result<Self, Error> {
        let repr = store.get(cid).await?;
        let self_ = repr.repr_to_owned()?;
        Ok(self_)
    }
    async fn save(&self, store: &S) -> Result<H::Cid, Error> {
        // TODO: Nix the ownership of `put()` - owning the type seems less likely these days.
        let cid = store.put(self.clone()).await?;
        Ok(cid)
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        fixity_store::store::{json_store::JsonStore, rkyv_store::RkyvStore},
        rstest::*,
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
        GCounter: Content<S>,
    {
        let mut counter = GCounter::default();
        counter.inc(0);
        let cid = counter.save(&store).await.unwrap();
        assert_eq!(GCounter::load(&store, &cid).await.unwrap(), counter);
    }
}
