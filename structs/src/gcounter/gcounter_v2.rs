use super::GCounterInt;
use fixity_store::{container::NewContainer, deser::Rkyv, replicaid::Rid, store::Repr};

pub struct GCounter<const N: usize>(Vec<(Rid<N>, GCounterInt)>);

impl<'s, const N: usize, S> NewContainer<'s, S> for GCounter<N> {}

// TODO: Convert Vec back to BTree for faster lookups? This was made a Vec
// due to difficulties in looking up `ArchivedRid`.
// Once `ArchivedRid` and `Rid` are unified into a single Rkyv-friendly type,
// in theory we can go back to a Rid.
pub struct GCounterRef<const N: usize, D = Rkyv>(Repr<Vec<(Rid<N>, GCounterInt)>, D>);
