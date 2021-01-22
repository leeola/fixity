use {
    super::{Error, Guard, Status, Workspace},
    crate::{head::Head, Addr},
    std::{ops::Drop, path::PathBuf},
};
const WORKSPACE_LOCK_FILE_NAME: &str = "WORKSPACE.lock";
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
    type Guard<'a> = FsGuard;
    async fn lock(&self) -> Result<Self::Guard<'_>, Error> {
        let file_lock_path = self
            .fixi_dir
            .join(&self.workspace)
            .join(WORKSPACE_LOCK_FILE_NAME);
        // using a non-async File since we're going to drop it in a blocking manner.
        let file_lock_res = std::fs::OpenOptions::new()
            .create_new(true)
            .open(&file_lock_path);
        let workspace_guard_file = match file_lock_res {
            Ok(f) => f,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(Error::InUse)
            }
            Err(err) => {
                return Err(Error::Internal(format!(
                    "failed to acquire workspace lock: {}",
                    err
                )))
            }
        };
        Ok(FsGuard {
            workspace_guard_file: Some(workspace_guard_file),
            file_lock_path,
        })
    }
    async fn status(&self) -> Result<Status, Error> {
        todo!("MemoryGuard")
    }
}
pub struct FsGuard {
    workspace_guard_file: Option<std::fs::File>,
    file_lock_path: PathBuf,
}
#[async_trait::async_trait]
impl Guard for FsGuard {
    async fn status(&self) -> Result<Status, Error> {
        todo!("FsGuard")
    }
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error> {
        todo!("FsGuard")
    }
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error> {
        todo!("FsGuard")
    }
}
impl Drop for FsGuard {
    fn drop(&mut self) {
        let _ = self.workspace_guard_file.take();
        if let Err(err) = std::fs::remove_file(&self.file_lock_path) {
            log::info!(
                "failed to release workspace lock. path:{:?}",
                self.file_lock_path
            )
        }
    }
}
