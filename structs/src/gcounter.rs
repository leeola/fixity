pub mod gcounter_v2;
pub mod owned_or_repr;

use self::owned_or_repr::{Oor, OwnedOrRepr};
use async_trait::async_trait;
use fixity_store::{
    deser::{Deserialize, Rkyv},
    replicaid::{ReplicaId, Rid},
    store::Repr,
};
use rkyv::{collections::ArchivedBTreeMap, vec::ArchivedVec};
use std::collections::{btree_map::Entry, BTreeMap};

type GCounterInt = u32;

#[derive(Clone, PartialEq, Eq)]
struct GCounter<const N: usize, D = Rkyv>(
    // NIT: Optionally use ProllyTree for large concurrent uses.
    // NIT: Make int generic to support multiple sizes?
    // TODO: Convert Vec back to BTree for faster lookups? This was made a Vec
    // due to difficulties in looking up `ArchivedRid`.
    // Once `ArchivedRid` and `Rid` are unified into a single Rkyv-friendly type,
    // in theory we can go back to a Rid.
    // Oor<BTreeMap<Rid<N>, GCounterInt>, D>,
    Oor<Vec<(Rid<N>, GCounterInt)>, D>,
);
impl<const N: usize> GCounter<N> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<const N: usize> GCounter<N, Rkyv> {
    pub fn inc(&mut self, rid: Rid<N>) {
        let owned = self.0.owned_as_mut().unwrap();
        let index_res = owned.binary_search_by_key(&&rid, |(rid, _)| rid);
        match index_res {
            Ok(i) => {
                let (_, count) = owned.get_mut(i).expect("index returned by `binary_search`");
            },
            Err(i) => owned.insert(i, (rid, 1)),
        }
    }
    pub fn value(&self) -> u32 {
        // TODO: cache the result.
        match self.0.inner() {
            OwnedOrRepr::Owned(values) => values.iter().map(|(_, i)| i).sum(),
            OwnedOrRepr::Repr(repr) => {
                let values = repr.repr_ref().unwrap();
                values.iter().map(|(_, i)| i).sum()
            },
        }
    }
    pub fn contains_rid(&self, rid: &Rid<N>) -> bool {
        match self.0.inner() {
            OwnedOrRepr::Owned(vals) => vals.binary_search_by_key(&rid, |(rid, _)| rid).is_ok(),
            OwnedOrRepr::Repr(repr) => {
                let vals = repr.repr_ref().unwrap();
                vals.binary_search_by(|(rhs, _)| rhs.partial_cmp(rid).unwrap())
                    .is_ok()
            },
        }
    }
    pub fn get(&self, rid: &Rid<N>) -> Option<GCounterInt> {
        match self.0.inner() {
            OwnedOrRepr::Owned(vals) => {
                let i = vals.binary_search_by_key(&rid, |(rid, _)| rid).ok()?;
                let (_, count) = vals.get(i).expect("index returned by `binary_search`");
                Some(*count)
            },
            OwnedOrRepr::Repr(repr) => {
                let vals = repr.repr_ref().unwrap();
                let i = vals
                    .binary_search_by(|(rhs, _)| rhs.partial_cmp(rid).unwrap())
                    .ok()?;
                let (_, count) = vals.get(i).expect("index returned by `binary_search`");
                Some(*count)
            },
        }
    }
    // pub fn iter(&self) -> impl Iterator<Item = (&Rid<N>, &GCounterInt)> {
    //     todo!()
    // }
    pub fn merge(&mut self, other: &Self) {
        let self_ = self.0.owned_as_mut().unwrap();
        match other.0.inner() {
            OwnedOrRepr::Owned(other) => {
                todo!()
                // let idx = other.binary_search_by_key(&rid, |(rid, _)| rid);
                // match idx {
                //     Ok(idx) => {
                // let other_i = other[i];
                // let self_i =
                //     todo!("write this cleanly without nesting the binary_search results");
                //     },
                // }
            },
            OwnedOrRepr::Repr(repr) => {
                todo!()
            },
        }
        // for (rid, other_i) in other.iter() {}
        /*
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
        */
        todo!()
    }
}
impl<const N: usize, D> Default for GCounter<N, D> {
    fn default() -> Self {
        Self(Oor::default())
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    /*
    use fixity_store::store::{json_store::JsonStore, rkyv_store::RkyvStore};
    use rstest::*;
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
    */
}
