use crate::cid::ContentId;

pub trait ReplicaId: ContentId {}
impl<const N: usize> ReplicaId for [u8; N] where [u8; N]: ContentId {}
