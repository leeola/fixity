use super::GCounterInt;
use async_trait::async_trait;
use fixity_store::{
    container::{ContainerRef, ContainerRefInto, ContainerV4},
    content_store::ContentStore,
    contentid::Cid,
    deser::Rkyv,
    deser_store::DeserStore,
    replicaid::Rid,
    store::StoreError,
    type_desc::{TypeDescription, ValueDesc},
};
use std::any::{self, TypeId};

type IVec = Vec<(Rid, GCounterInt)>;

#[derive(Debug)]
pub struct GCounter(IVec);
impl GCounter {
    pub fn new() -> Self {
        Self(IVec::default())
    }
}
impl GCounter {
    pub fn inc(&mut self, rid: Rid) {
        let self_ = &mut self.0;
        let idx_result = self_.binary_search_by_key(&&rid, |(rid, _)| rid);
        match idx_result {
            Ok(idx) => {
                let (_, count) = self_
                    .get_mut(idx)
                    .expect("index returned by `binary_search`");
                *count += 1;
            },
            Err(idx) => self_.insert(idx, (rid, 1)),
        }
        debug_assert!(self_.windows(2).all(|w| w[0] <= w[1]));
    }
    pub fn value(&self) -> GCounterInt {
        // TODO: cache the result.
        self.0.iter().map(|(_, i)| i).sum()
    }
    pub fn get(&self, rid: &Rid) -> Option<GCounterInt> {
        let i = self.0.binary_search_by_key(&rid, |(rid, _)| rid).ok()?;
        let (_, count) = self.0.get(i).expect("index returned by `binary_search`");
        Some(*count)
    }
}
impl TypeDescription for GCounter {
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: any::type_name::<Self>(),
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<IVec>()],
        }
    }
}
#[async_trait]
impl<S> ContainerV4<S> for GCounter
where
    S: ContentStore,
{
    fn deser_type_desc() -> ValueDesc {
        Self::type_desc()
    }
    fn new_container(_: &S) -> Self {
        Self::new()
    }
    async fn open(store: &S, cid: &Cid) -> Result<Self, StoreError> {
        let repr = store.get::<IVec>(cid).await?;
        let inner = repr.repr_to_owned()?;
        Ok(Self(inner))
    }
    async fn save(&mut self, store: &S) -> Result<Cid, StoreError> {
        store.put::<IVec>(&self.0).await
    }
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        store.put_with_cids::<IVec>(&self.0, cids_buf).await
    }
    async fn merge(&mut self, store: &S, other: &Cid) -> Result<(), StoreError> {
        let other = {
            let repr = store.get::<IVec>(other).await?;
            repr.repr_to_owned()?
        };
        let mut start_idx = 0;
        for (other_rid, other_value) in other {
            if start_idx >= self.0.len() {
                self.0.push((other_rid, other_value));
                continue;
            }
            // Assume both are sorted, nearby debug_assert helps validate.
            let idx = self.0[start_idx..].binary_search_by_key(&&other_rid, |(rid, _)| rid);
            let idx = match idx {
                Ok(idx) => {
                    let (_, self_value) = &mut self.0[idx];
                    if other_value > *self_value {
                        *self_value = other_value;
                    }
                    idx
                },
                Err(idx) => {
                    self.0.insert(idx, (other_rid, other_value));
                    idx
                },
            };
            start_idx = idx + 1;
        }
        debug_assert!(self.0.windows(2).all(|w| w[0] <= w[1]));
        Ok(())
    }
    async fn diff(&mut self, _store: &S, _other: &Cid) -> Result<Self, StoreError> {
        todo!()
    }
}
#[cfg(test)]
pub mod test {
    use fixity_store::{replicaid::Rid, stores::memory::Memory};

    use super::*;
    #[tokio::test]
    async fn poc() {
        let store = Memory::test();
        let mut a = GCounter::<Rid>::default();
        a.inc(1.into());
        assert_eq!(a.value(), 1);
        a.inc(1.into());
        a.inc(0.into());
        assert_eq!(a.value(), 3);
        let mut b = GCounter::<Rid>::default();
        b.inc(1.into());
        b.inc(1.into());
        let b_cid = b.save(&store).await.unwrap();
        a.merge(&store, &b_cid).await.unwrap();
        assert_eq!(a.value(), 3);
        b.inc(1.into());
        let b_cid = b.save(&store).await.unwrap();
        a.merge(&store, &b_cid).await.unwrap();
        assert_eq!(a.value(), 4);
    }
}
