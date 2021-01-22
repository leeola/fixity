mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::Addr;
#[async_trait::async_trait]
pub trait Workspace: Sized {
    // WARN: Dragons ahead.. using GATs.. :shock:
    // Might move away from using GATs here, though i'm unsure the solution offhand.
    // I really hate not being non-stable in this repo, but .. it may be worth it for this.
    // For now it makes the code clean, and this repo is about experimenting.
    // So.. lets find out. :sus:
    type Guard<'a>: Guard;
    async fn lock(&self) -> Result<Self::Guard<'_>, Error> {
        todo!("tmp auto impl")
    }
    // async fn status(&self) -> Result<Status, Error>;
    // async fn log(&self) -> Result<Log, Error>;
    // async fn metalog(&self) -> Result<MetaLog, Error>;
}
#[async_trait::async_trait]
pub trait Guard {
    async fn status(&self) -> Result<Status, Error> {
        todo!("tmp auto impl")
    }
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error> {
        todo!("tmp auto impl")
    }
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error> {
        todo!("tmp auto impl")
    }
}
#[derive(Debug, Clone)]
pub enum Status {
    Init {
        branch: String,
    },
    InitStaged {
        staged: Addr,
        branch: String,
    },
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
    #[error("workspace in use")]
    InUse,
}
