mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::{primitive::CommitLog, storage::StorageRead, Addr};
#[async_trait::async_trait]
pub trait Workspace: Sized {
    // WARN: Dragons ahead.. using GATs.. :shock:
    // Might move away from using GATs here, though i'm unsure the solution offhand.
    // I really hate not being non-stable in this repo, but .. it may be worth it for this.
    // For now it makes the code clean, and this repo is about experimenting.
    // So.. lets find out. :sus:
    type Guard<'a>: Guard;
    async fn lock(&self) -> Result<Self::Guard<'_>, Error>;
    // async fn log(&self) -> Result<Log, Error>;
    // async fn metalog(&self) -> Result<MetaLog, Error>;
    async fn status(&self) -> Result<Status, Error>;
}
#[async_trait::async_trait]
pub trait Guard {
    async fn status(&self) -> Result<Status, Error>;
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error>;
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error>;
}
#[derive(Debug, Clone)]
pub enum Status {
    Init {
        branch: String,
    },
    InitStaged {
        branch: String,
        staged_content: Addr,
    },
    Detached(Addr),
    Clean {
        branch: String,
        commit: Addr,
    },
    Staged {
        branch: String,
        staged_content: Addr,
        commit: Addr,
    },
}
impl Status {
    /// Return the underlying commit address, if available.
    pub fn commit_addr(&self) -> Option<Addr> {
        match self {
            Self::Detached(commit) | Self::Clean { commit, .. } | Self::Staged { commit, .. } => {
                Some(commit.clone())
            }
            _ => None,
        }
    }
    /// Return the underlying staged _content_ address, if available.
    pub fn staged_addr(&self) -> Option<Addr> {
        match self {
            Self::InitStaged { staged_content, .. } | Self::Staged { staged_content, .. } => {
                Some(staged_content.clone())
            }
            _ => None,
        }
    }
    /// Resolve the content address
    // NIT: Not sure where best to put this helper. Not a fan of it on `Status`.
    pub async fn content_addr<S>(&self, storage: &S) -> Result<Option<Addr>, crate::Error>
    where
        S: StorageRead,
    {
        match self {
            Status::Init { .. } => Ok(None),
            Status::InitStaged { staged_content, .. } | Status::Staged { staged_content, .. } => {
                Ok(Some(staged_content.clone()))
            }
            Status::Detached(_) => return Err(crate::Error::DetachedHead),
            Status::Clean { commit, .. } => {
                let commit_log = CommitLog::new(storage, Some(commit.clone()));
                let commit =
                    commit_log
                        .first()
                        .await?
                        .ok_or_else(|| crate::Error::DanglingAddr {
                            message: "commit HEAD".to_owned(),
                            addr: Some(commit.clone()),
                        })?;
                Ok(Some(commit.content))
            }
        }
    }
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
