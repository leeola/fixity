use {
    super::Workspace,
    crate::{head::Head, Error},
    std::path::PathBuf,
};
pub struct Memory {
    workspace: String,
}
impl Memory {
    pub fn new(workspace: String) -> Self {
        Self { workspace }
    }
}
impl Workspace for Memory {}
