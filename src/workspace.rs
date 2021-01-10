mod fs;
mod memory;
pub use self::{fs::Fs, memory::Memory};
#[async_trait::async_trait]
pub trait Workspace: Sized {
    // Possibly useful for switching/creating workspaces.
    // async fn init(&self, workspace: String) -> Result<Self, Error>;
    // async fn open(&self, workspace: String) -> Result<Self, Error>;
}
