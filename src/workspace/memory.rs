use {
    super::{Error, Guard, Status, Workspace, Workspace2},
    crate::Addr,
    std::{
        collections::HashMap,
        sync::{Mutex, MutexGuard},
    },
};
#[derive(Debug, Clone)]
pub(super) enum HeadState {
    Init,
    InitStaged { staged: Addr },
    Detached(Addr),
    Clean,
    Staged { staged: Addr },
}
pub struct Memory(Mutex<InnerMemory>);
struct InnerMemory {
    head: HeadState,
    branch: String,
    branches: HashMap<String, Addr>,
}
impl Memory {
    pub fn new(_workspace: String) -> Self {
        Self(Mutex::new(InnerMemory {
            head: HeadState::Init,
            branch: "default".to_owned(),
            branches: HashMap::new(),
        }))
    }
}
pub struct MemoryGuard<'a>(MutexGuard<'a, InnerMemory>);
#[async_trait::async_trait]
impl Workspace2 for Memory {
    type Guard = MemoryGuard<'a>;
}
#[async_trait::async_trait]
impl Guard for MemoryGuard {}
#[async_trait::async_trait]
impl Workspace for Memory {
    /*
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
    */
}
