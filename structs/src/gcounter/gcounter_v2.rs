use super::GCounterInt;
use async_trait::async_trait;
use fixity_store::{
    container::{ContainerRef, NewContainer},
    contentid::NewContentId,
    deser::{Deserialize, Rkyv, Serialize},
    deser_store::DeserStore,
    replicaid::{NewReplicaId, ReplicaIdDeser, Rid},
    store::{Repr, StoreError},
    type_desc::{TypeDescription, ValueDesc},
    Store,
};

pub struct GCounter<Rid>(Vec<(Rid, GCounterInt)>);
impl<Rid> TypeDescription for GCounter<Rid>
where
    Rid: NewReplicaId,
{
    fn type_desc() -> ValueDesc {
        todo!()
    }
}
#[async_trait]
impl<'s, Rid, Cid> NewContainer<Rkyv, Cid> for GCounter<Rid>
where
    Cid: NewContentId,
    Rid: ReplicaIdDeser<Rkyv>,
{
    fn deser_type_desc() -> ValueDesc {
        todo!()
    }
    async fn open<S: DeserStore<Rkyv, Cid>>(store: &S, cid: &Cid) -> Result<Self, StoreError> {
        todo!()
    }
    async fn save<S: DeserStore<Rkyv, Cid>>(&mut self, store: &S) -> Result<Cid, StoreError> {
        todo!()
    }
    async fn save_with_cids<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<Cid>,
    ) -> Result<(), StoreError> {
        todo!()
    }
    async fn merge<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<(), StoreError> {
        todo!()
    }
    async fn diff<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<Self, StoreError> {
        todo!()
    }
}
#[async_trait]
impl<'s, Rid, Cid> ContainerRef<Rkyv, Cid> for GCounter<Rid>
where
    Cid: NewContentId,
    Rid: ReplicaIdDeser<Rkyv>,
{
    type Ref = GCounterRef<Rid, Rkyv>;
    type DiffRef = GCounterRef<Rid, Rkyv>;
    async fn open_ref<S: DeserStore<Rkyv, Cid>>(
        store: &S,
        cid: &Cid,
    ) -> Result<Self::Ref, StoreError> {
        todo!()
    }
    async fn diff_ref<S: DeserStore<Rkyv, Cid>>(
        &mut self,
        store: &S,
        other: &Cid,
    ) -> Result<Self::DiffRef, StoreError> {
        todo!()
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
