mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
use crate::{Addr, Error};
#[async_trait::async_trait]
pub trait Workspace: Sized {
    // Possibly useful for switching/creating workspaces.
    // async fn init(&self, workspace: String) -> Result<Self, Error>;
    // async fn open(&self, workspace: String) -> Result<Self, Error>;
    async fn head(&self) -> Result<Option<Addr>, Error>;
    // async fn move_head(&self, addr: Addr) -> Result<(), Error>;
}
