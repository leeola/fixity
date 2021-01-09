use {
    crate::{head::Head, Error},
    std::path::PathBuf,
};
pub struct Workspace {
    fixi_dir: PathBuf,
    workspace: String,
}
impl Workspace {
    pub async fn init(fixi_dir: PathBuf, workspace: String) -> Result<Self, Error> {
        let _ = Head::init(fixi_dir.as_path(), workspace.as_str()).await?;
        Ok(Self {
            fixi_dir,
            workspace,
        })
    }
    pub async fn open(fixi_dir: PathBuf, workspace: String) -> Result<Self, Error> {
        Ok(Self {
            fixi_dir,
            workspace,
        })
    }
}
