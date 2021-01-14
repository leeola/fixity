use {
    super::Workspace,
    crate::{Addr, Error},
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
            .map_err(|_| Error::Unhandled("failed to acquire workspace lock".into()))?;
        let addr = match &inner.head {
            HeadState::Detached(addr) => Some(addr.clone()),
            HeadState::Branch(branch) => inner.branches.get(branch).cloned(),
        };
        Ok(addr)
    }
}
