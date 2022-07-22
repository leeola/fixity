use super::GCounterInt;
use async_trait::async_trait;
use fixity_store::{
    container::NewContainer,
    deser::Rkyv,
    replicaid::Rid,
    store::{Repr, StoreError},
    Store,
};

pub struct GCounter<const N: usize>(Vec<(Rid<N>, GCounterInt)>);

#[async_trait]
impl<'s, const N: usize, S> NewContainer<'s, S> for GCounter<N>
where
    S: Store,
{
    type Ref = GCounterRef<N, S::Deser>;
    async fn open(
        store: &'s S,
        cid: &<S as Store>::Cid,
    ) -> Result<Self, fixity_store::store::StoreError> {
        todo!()
    }
    async fn open_ref(
        store: &'s S,
        cid: &<S as Store>::Cid,
    ) -> Result<Self::Ref, fixity_store::store::StoreError> {
        todo!()
    }
    async fn save(
        &mut self,
        store: &'s S,
    ) -> Result<<S as Store>::Cid, fixity_store::store::StoreError> {
        todo!()
    }
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<<S as Store>::Cid>,
    ) -> Result<(), fixity_store::store::StoreError> {
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
