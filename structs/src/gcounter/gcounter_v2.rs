use super::GCounterInt;
use async_trait::async_trait;
use fixity_store::{
    container::{ContainerRef, NewContainer},
    contentid::NewContentId,
    deser::{Deserialize, Rkyv, Serialize},
    deser_store::DeserStore,
    replicaid::NewReplicaId,
    store::{Repr, StoreError},
    type_desc::{TypeDescription, ValueDesc},
};
use std::any::TypeId;

type Ints<Rid> = Vec<(Rid, GCounterInt)>;

#[derive(Debug)]
pub struct GCounter<Rid>(Ints<Rid>);
impl<Rid> GCounter<Rid> {
    pub fn new() -> Self {
        Self(Ints::default())
    }
}
impl<Rid: NewReplicaId> GCounter<Rid> {
    pub fn inc(&mut self, rid: Rid) {
        let self_ = &mut self.0;
        let index_res = self_.binary_search_by_key(&&rid, |(rid, _)| rid);
        match index_res {
            Ok(i) => {
                let (_, count) = self_.get_mut(i).expect("index returned by `binary_search`");
                *count += 1;
            },
            Err(i) => self_.insert(i, (rid, 1)),
        }
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
impl<Rid> TypeDescription for GCounter<Rid>
where
    Rid: NewReplicaId,
{
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "GCounter",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<Ints<Rid>>()],
        }
    }
}
#[async_trait]
impl<'s, Rid, Cid> NewContainer<Rkyv, Cid> for GCounter<Rid>
where
    Cid: NewContentId,
    Rid: NewReplicaId,
    Ints<Rid>: Serialize<Rkyv> + Deserialize<Rkyv>,
{
    fn deser_type_desc() -> ValueDesc {
        Self::type_desc()
    }
    fn new_container<S: DeserStore<Rkyv, Cid>>(_: &S) -> Self {
        Self::new()
    }
    async fn open<S: DeserStore<Rkyv, Cid>>(store: &S, cid: &Cid) -> Result<Self, StoreError> {
        let repr = store.get::<Ints<Rid>>(cid).await?;
        let inner = repr.repr_to_owned()?;
        Ok(Self(inner))
    }
    async fn save<S: DeserStore<Rkyv, Cid>>(&mut self, store: &S) -> Result<Cid, StoreError> {
        store.put::<Ints<Rid>>(&self.0).await
    }
    async fn save_with_cids<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        store.put_with_cids::<Ints<Rid>>(&self.0, cids_buf).await
    }
    async fn merge<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<(), StoreError> {
        let other = {
            let repr = store.get::<Ints<Rid>>(other).await?;
            repr.repr_to_owned()?
        };
        for (other_rid, other_value) in other {
            let idx = self.0.binary_search_by_key(&&other_rid, |(rid, _)| rid);
            match idx {
                Ok(idx) => {
                    let (_, self_value) = &mut self.0[idx];
                    if other_value > *self_value {
                        *self_value = other_value;
                    }
                },
                Err(idx) => {
                    self.0.insert(idx, (other_rid, other_value));
                },
            }
        }
        Ok(())
    }
    async fn diff<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        _store: &S,
        _other: &Cid,
    ) -> Result<Self, StoreError> {
        todo!()
    }
}
#[async_trait]
impl<'s, Rid, Cid> ContainerRef<Rkyv, Cid> for GCounter<Rid>
where
    Cid: NewContentId,
    Rid: NewReplicaId,
    Ints<Rid>: Serialize<Rkyv> + Deserialize<Rkyv>,
{
    type Ref = GCounterRef<Rid, Rkyv>;
    type DiffRef = GCounterRef<Rid, Rkyv>;
    async fn open_ref<S: DeserStore<Rkyv, Cid>>(
        _store: &S,
        _cid: &Cid,
    ) -> Result<Self::Ref, StoreError> {
        todo!()
    }
    async fn diff_ref<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        _store: &S,
        _other: &Cid,
    ) -> Result<Self::DiffRef, StoreError> {
        todo!()
    }
}
impl<Rid> Default for GCounter<Rid> {
    fn default() -> Self {
        Self::new()
    }
}
// TODO: Convert Vec back to BTree for faster lookups? This was made a Vec
// due to difficulties in looking up `ArchivedRid`.
// Once `ArchivedRid` and `Rid` are unified into a single Rkyv-friendly type,
// in theory we can go back to a Rid.
pub struct GCounterRef<Rid, D>(Repr<Vec<(Rid, GCounterInt)>, D>);
impl<Rid, D> TryInto<GCounter<Rid>> for GCounterRef<Rid, D> {
    type Error = StoreError;
    fn try_into(self) -> Result<GCounter<Rid>, Self::Error> {
        todo!()
    }
}
#[cfg(test)]
pub mod test {
    use fixity_store::stores::memory::Memory;

    use super::*;
    #[tokio::test]
    async fn poc() {
        let mut a = GCounter::default();
        a.inc(1);
        assert_eq!(a.value(), 1);
        a.inc(1);
        a.inc(0);
        assert_eq!(a.value(), 3);
        let mut b = GCounter::default();
        b.inc(1);
        b.inc(1);
        let store = Memory::test();
        let cid = b.save(&store).await.unwrap();
        // b.merge(&a);
        // assert_eq!(b.value(), 4);
    }
}
