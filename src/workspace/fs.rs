use {
    super::{Error, Workspace},
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
    async fn head(&self) -> Result<Option<Addr>, Error> {
        todo!("workspace fs head")
    }
    async fn append(&self, _addr: Addr) -> Result<(), Error> {
        todo!("workspace fs append")
    }
}
