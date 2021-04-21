use crate::{Addr, Error};
#[async_trait::async_trait]
pub trait SegmentReplace<C>: Sized {
    async fn replace(
        self,
        storage: &C,
        self_addr: Option<Addr>,
        child_addr: Addr,
    ) -> Result<(Self, Addr), Error>;
}
