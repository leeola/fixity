mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::{
    core::{cache::CacheRead, primitive::CommitLog},
    Addr,
};
#[async_trait::async_trait]
pub trait Init {
    type Workspace: Workspace;
    async fn init(&self, workspace: String) -> Result<Self::Workspace, Error>;
    async fn open(&self, workspace: String) -> Result<Self::Workspace, Error>;
}
#[async_trait::async_trait]
pub trait Workspace: Send + Sync {
    type Guard<'a>: Guard;
    async fn lock(&self) -> Result<Self::Guard<'_>, Error>;
    // async fn log(&self) -> Result<Log, Error>;
    // async fn metalog(&self) -> Result<MetaLog, Error>;
    async fn status(&self) -> Result<Status, Error>;
}
#[async_trait::async_trait]
pub trait Guard: Send + Sync {
    async fn status(&self) -> Result<Status, Error>;
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error>;
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error>;
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("cannot commit empty STAGE")]
    CommitEmptyStage,
    #[error("cannot commit or stage on a detatched HEAD")]
    DetachedHead,
    #[error("workspace in use")]
    InUse,
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
            },
            _ => None,
        }
    }
    /// Return the underlying staged _content_ address, if available.
    pub fn staged_addr(&self) -> Option<Addr> {
        match self {
            Self::InitStaged { staged_content, .. } | Self::Staged { staged_content, .. } => {
                Some(staged_content.clone())
            },
            _ => None,
        }
    }
    /// Resolve the content address
    // NIT: Not sure where best to put this helper. Not a fan of it on `Status`.
    pub async fn content_addr<C>(&self, storage: &C) -> Result<Option<Addr>, crate::Error>
    where
        C: CacheRead,
    {
        match self {
            Status::Init { .. } => Ok(None),
            Status::InitStaged { staged_content, .. } | Status::Staged { staged_content, .. } => {
                Ok(Some(staged_content.clone()))
            },
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
            },
        }
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, proptest::prelude::*, std::convert::TryFrom, tokio::runtime::Runtime};
    #[derive(Debug, Copy, Clone)]
    enum TestWorkspace {
        Memory,
        Fs,
    }
    #[derive(Debug, Copy, Clone)]
    enum TestAction {
        Stage,
        Commit,
    }
    fn general_behavior_inputs(
        workspaces: impl Strategy<Value = TestWorkspace> + Clone,
    ) -> impl Strategy<Value = (TestWorkspace, Vec<Addr>, Vec<TestAction>)> {
        (1..10usize).prop_flat_map(move |change_count| {
            (
                workspaces.clone(),
                prop::collection::vec(
                    prop::collection::vec(0u8..u8::MAX, Addr::LEN)
                        .prop_map(|bytes| Addr::try_from(bytes).unwrap()),
                    change_count,
                ),
                prop::collection::vec(
                    (0..2usize).prop_map(|i| match i {
                        0 => TestAction::Stage,
                        1 => TestAction::Commit,
                        _ => unreachable!(),
                    }),
                    change_count,
                ),
            )
        })
    }
    proptest! {
        #[test]
        fn mem_general_behavior(
            (_workspace, addrs, test_actions) in general_behavior_inputs(
                (0..1usize)
                    .prop_map(|i| match i {
                        0 => TestWorkspace::Memory,
                        _ => unreachable!(),
                    })),
            ) {
            Runtime::new().unwrap().block_on(async {
                let workspace = Memory::new("default".to_string());
                test_general_behavior(workspace, &addrs, &test_actions).await;
            });
        }
        #[test]
        #[ignore]
        fn fs_general_behavior(
            (_workspace, addrs, test_actions) in general_behavior_inputs(
                (0..1usize)
                    .prop_map(|i| match i {
                        0 => TestWorkspace::Fs,
                        _ => unreachable!(),
                    })),
            ) {
            Runtime::new().unwrap().block_on(async {
                // TODO: I could/should incorporate init/open behavior into these tests.
                // Which should also include RNG workspace names.
                let temp_dir = tempfile::tempdir().unwrap();
                let workspaces_path = temp_dir.path().to_owned().join("workspaces");
                let workspace = Fs::init(workspaces_path, "default".to_string()).await.unwrap();
                test_general_behavior(workspace, &addrs, &test_actions).await;
            });
        }
    }
    async fn test_general_behavior<W: Workspace>(
        workspace: W,
        addrs: &[Addr],
        test_actions: &[TestAction],
    ) {
        let mut prev_status = workspace.status().await.unwrap();
        assert!(matches!(prev_status, Status::Init { .. }));
        let guard = workspace.lock().await.unwrap();
        for (addr, test_action) in addrs.iter().cloned().zip(test_actions.iter()) {
            match test_action {
                TestAction::Stage => match prev_status {
                    Status::Init { .. } | Status::InitStaged { .. } => {
                        guard.stage(addr.clone()).await.unwrap();
                        let new_status = guard.status().await.unwrap();
                        assert!(matches!(new_status, Status::InitStaged { .. }));
                        assert_eq!(new_status.staged_addr().unwrap(), addr);
                        prev_status = new_status
                    },
                    Status::Clean { .. } | Status::Staged { .. } => {
                        guard.stage(addr.clone()).await.unwrap();
                        let new_status = guard.status().await.unwrap();
                        assert!(matches!(new_status, Status::Staged { .. }));
                        assert_eq!(new_status.staged_addr().unwrap(), addr);
                        prev_status = new_status
                    },
                    Status::Detached(_) => unreachable!("action not implemented in tests yet"),
                },
                TestAction::Commit => match prev_status {
                    Status::Init { .. } => {
                        assert!(matches!(
                            guard.commit(addr).await,
                            Err(Error::CommitEmptyStage)
                        ));
                    },
                    Status::InitStaged { .. } | Status::Staged { .. } => {
                        guard.commit(addr.clone()).await.unwrap();
                        let new_status = guard.status().await.unwrap();
                        assert!(matches!(new_status, Status::Clean { .. }));
                        assert_eq!(new_status.commit_addr().unwrap(), addr);
                        prev_status = new_status
                    },
                    Status::Clean { .. } => {
                        assert!(matches!(
                            guard.commit(addr).await,
                            Err(Error::CommitEmptyStage)
                        ));
                    },
                    Status::Detached(_) => unreachable!("action not implemented in tests yet"),
                },
            }
        }
    }
}
/// A helper trait to allow a single `T` to return references to both a `Workspace` and
/// a `Cache`.
///
/// See [`Commit`](crate::Commit) for example usage.
pub trait AsWorkspaceRef {
    type Workspace: Workspace;
    fn as_workspace_ref(&self) -> &Self::Workspace;
}
