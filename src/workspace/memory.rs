use {
    super::{Error, Workspace},
    crate::Addr,
    std::{collections::HashMap, sync::Mutex},
};
pub struct Memory(Mutex<InnerMemory>);
struct InnerMemory {
    head: HeadState,
    branches: HashMap<String, Addr>,
}
enum HeadState {
    Detached(Addr),
    Branch(String),
}
impl Memory {
    pub fn new(_workspace: String) -> Self {
        Self(Mutex::new(InnerMemory {
            head: HeadState::Branch("default".to_owned()),
            branches: HashMap::new(),
        }))
    }
}
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
}
