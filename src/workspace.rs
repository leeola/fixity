mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::Addr;
#[async_trait::async_trait]
pub trait Workspace: Sized {
    async fn mutate(&self) -> Result<(), Error> {
        todo!("tmp auto impl")
    }
    // async fn status(&self) -> Result<Status, Error>;
    // async fn log(&self) -> Result<Log, Error>;
    // async fn metalog(&self) -> Result<MetaLog, Error>;
}
#[async_trait::async_trait]
pub trait Workspace2: Sized {
    type Guard: Guard;
    async fn lock(&self) -> Result<Box<dyn Guard>, Error> {
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
    async fn stage(&mut self, stage_addr: Addr) -> Result<(), Error> {
        todo!("tmp auto impl")
    }
    async fn commit(&mut self, commit_addr: Addr) -> Result<(), Error> {
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
}
