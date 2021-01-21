use {
    super::{Error, Status, Workspace},
    crate::{head::Head, Addr},
    std::path::PathBuf,
};
pub struct Fs {
    fixi_dir: PathBuf,
    workspace: String,
}
impl Fs {
    pub async fn init(fixi_dir: PathBuf, workspace: String) -> Result<Self, Error> {
        let _ = Head::init(fixi_dir.as_path(), workspace.as_str())
            .await
            .map_err(|err| {
                // TODO: convert Head to use a workspace error.
                Error::Internal(format!("{}", err))
            })?;
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
#[async_trait::async_trait]
impl Workspace for Fs {
    /*
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error> {
        todo!("workspace fs stage")
    }
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error> {
        todo!("workspace fs commit")
    }
    async fn status(&self) -> Result<Status, Error> {
        todo!("workspace fs status")
    }
    */
}
