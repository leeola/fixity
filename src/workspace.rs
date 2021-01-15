mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::Addr;
#[async_trait::async_trait]
pub trait Workspace: Sized {
    async fn head(&self) -> Result<Option<Addr>, Error>;
    async fn append(&self, addr: Addr) -> Result<(), Error>;
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("cannot commit empty STAGE")]
    CommitEmptyStage,
    #[error("cannot commit or stage on a detatched HEAD")]
    DetatchedHead,
}
