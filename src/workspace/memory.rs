use {
    super::{Error, Guard, Status, Workspace},
    crate::Addr,
    std::{
        collections::HashMap,
        mem,
        sync::{Arc, Mutex, MutexGuard},
    },
};
#[derive(Debug, Clone)]
pub(super) enum HeadState {
    Init { branch: String },
    InitStaged { branch: String, staged: Addr },
    Detached(Addr),
    Clean { branch: String },
    Staged { branch: String, staged: Addr },
    Aborted,
}
pub struct Memory {
    guard: Mutex<()>,
    state: Arc<Mutex<InnerMemory>>,
}
struct InnerMemory {
    head: HeadState,
    branches: HashMap<String, Addr>,
}
impl Memory {
    pub fn new(_workspace: String) -> Self {
        Self {
            guard: Mutex::new(()),
            state: Arc::new(Mutex::new(InnerMemory {
                head: HeadState::Init {
                    branch: "default".to_owned(),
                },
                branches: HashMap::new(),
            })),
        }
    }
}
#[async_trait::async_trait]
impl Workspace for Memory {
    type Guard<'a> = MemoryGuard<'a>;
    async fn lock(&self) -> Result<Self::Guard<'_>, Error> {
        let _guard = self
            .guard
            .lock()
            .map_err(|_| Error::Internal("failed to acquire workspace lock".into()))?;
        Ok(MemoryGuard {
            _guard,
            state: self.state.clone(),
        })
    }
}
pub struct MemoryGuard<'a> {
    _guard: MutexGuard<'a, ()>,
    state: Arc<Mutex<InnerMemory>>,
}
#[async_trait::async_trait]
impl<'a> Guard for MemoryGuard<'a> {
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error> {
        let mut inner = self
            .state
            .lock()
            .map_err(|_| Error::Internal("failed to acquire workspace lock".into()))?;
        if matches!(inner.head, HeadState::Detached(_)) {
            return Err(Error::DetatchedHead);
        }
        inner.head = match mem::replace(&mut inner.head, HeadState::Aborted) {
            HeadState::Init { branch } | HeadState::InitStaged { branch, .. } => {
                HeadState::InitStaged {
                    branch,
                    staged: stage_addr,
                }
            }
            HeadState::Clean { branch } | HeadState::Staged { branch, .. } => HeadState::Staged {
                branch,
                staged: stage_addr,
            },
            HeadState::Detached(_) => unreachable!("detached state checked above"),
            HeadState::Aborted => return Err(Error::Internal("HeadState::Aborted".into())),
        };
        Ok(())
    }
}
/*
#[async_trait::async_trait]
impl Workspace for Memory {
    async fn head(&self) -> Result<Option<Addr>, Error> {
        let inner = self
            .0
            .lock()
            .map_err(|_| Error::Internal("failed to acquire workspace lock".into()))?;
        let addr = match &inner.head {
            HeadState::Detached(addr) => Some(addr.clone()),
            HeadState::Branch(branch) => inner.branches.get(branch).cloned(),
        };
        Ok(addr)
    }
    async fn append(&self, addr: Addr) -> Result<(), Error> {
        use std::ops::{Deref, DerefMut};
        let mut inner = self
            .0
            .lock()
            .map_err(|_| Error::Internal("failed to acquire workspace lock".into()))?;
        let InnerMemory {
            ref head,
            ref mut branches,
        } = inner.deref_mut();
        // let head = &inner.head;
        // let branches = &mut inner.branches;
        let branch = match &head {
            HeadState::Detached(_) => return Err(Error::DetatchedHead),
            HeadState::Branch(branch) => branch,
        };
        if branches.contains_key(branch) {
            *branches.get_mut(branch).expect("impossibly missing") = addr;
        } else {
            branches.insert(branch.clone(), addr);
        }
        Ok(())
    }
    */
/*
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error> {
        let mut inner = self
            .0
            .lock()
            .map_err(|_| Error::Internal("failed to acquire workspace lock".into()))?;
        if matches!(inner.head, HeadState::Detached(_)) {
            return Err(Error::DetatchedHead);
        }
        inner.head = match inner.head {
            HeadState::Init | HeadState::InitStaged { .. } => {
                HeadState::InitStaged { staged: stage_addr }
            }
            HeadState::Clean | HeadState::Staged { .. } => HeadState::Staged { staged: stage_addr },
            HeadState::Detached(_) => unreachable!("detached state checked above"),
        };
        Ok(())
    }
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error> {
        let mut inner = self
            .0
            .lock()
            .map_err(|_| Error::Internal("failed to acquire workspace lock".into()))?;
        if matches!(inner.head, HeadState::Detached(_)) {
            return Err(Error::DetatchedHead);
        }
        if matches!(inner.head, HeadState::Init | HeadState::Clean) {
            return Err(Error::CommitEmptyStage);
        }
        let branch = inner.branch.clone();
        inner.branches.insert(branch, commit_addr);
        Ok(())
    }
    async fn status(&self) -> Result<Status, Error> {
        todo!("workspace mem status")
    }
}
    */
