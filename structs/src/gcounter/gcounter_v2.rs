use super::GCounterInt;
use async_trait::async_trait;
use fixity_store::{
    container::{ContainerRef, NewContainer},
    contentid::NewContentId,
    deser::Rkyv,
    deser_store::DeserStore,
    replicaid::Rid,
    store::{Repr, StoreError},
    type_desc::{TypeDescription, ValueDesc},
    Store,
};

pub struct GCounter<const N: usize>(Vec<(Rid<N>, GCounterInt)>);
impl<const N: usize> TypeDescription for GCounter<N> {
    fn type_desc() -> ValueDesc {
        todo!()
    }
}
#[async_trait]
impl<'s, const N: usize, Cid: NewContentId> NewContainer<Rkyv, Cid> for GCounter<N> {
    fn deser_desc() -> ValueDesc {
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
impl<'s, const N: usize, Cid: NewContentId> ContainerRef<Rkyv, Cid> for GCounter<N> {
    type Ref = GCounterRef<N, Rkyv>;
    type DiffRef = GCounterRef<N, Rkyv>;
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
pub struct GCounterRef<const N: usize, D>(Repr<Vec<(Rid<N>, GCounterInt)>, D>);

impl<const N: usize, D> TryInto<GCounter<N>> for GCounterRef<N, D> {
    type Error = StoreError;
    fn try_into(self) -> Result<GCounter<N>, Self::Error> {
        todo!()
    }
}
