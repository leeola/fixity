mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::Addr;
#[async_trait::async_trait]
pub trait Workspace: Sized {
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error>;
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error>;
    async fn status(&self) -> Result<Status, Error>;
    // async fn log(&self) -> Result<Log, Error>;
    // async fn metalog(&self) -> Result<MetaLog, Error>;
}
#[derive(Debug)]
pub enum Status {
    Init,
    Detached(Addr),
    Clean {
        branch: String,
        commit: Addr,
    },
    Staged {
        staged: Addr,
        branch: String,
        commit: Addr,
    },
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
