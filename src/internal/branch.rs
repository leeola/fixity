use {
    crate::{fixity::Flush, Addr},
    std::{
        path::{Path, PathBuf},
        str::FromStr,
    },
    tokio::{
        fs::{File, OpenOptions},
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
/// The internal folder where branch HEADs are stored.
const BRANCHES_DIR: &str = "branches";
pub struct Branch {
    branch_path: PathBuf,
    addr: Addr,
}
impl Branch {
    /// Create a new branch at the specified [`Addr`].
    ///
    /// # Errors
    ///
    /// If the branch already exists.
    pub async fn create<P, S>(workspace_path: P, branch_name: S, addr: Addr) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let branch_path = workspace_path.as_ref().join(branch_name.as_ref());
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&branch_path)
            .await
            .map_err(|err| Error::Io {
                path: branch_path.clone(),
                message: format!("create branch: {}", err),
            })?;
        f.write_all(addr.as_bytes())
            .await
            .map_err(|err| Error::Io {
                path: branch_path.clone(),
                message: format!("writing branch: {}", err),
            })?;
        f.sync_all().await.map_err(|err| Error::Io {
            path: branch_path.clone(),
            message: format!("syncing branch: {}", err),
        })?;
        Ok(Self { branch_path, addr })
    }
    /// Open an existing branch.
    ///
    /// # Errors
    ///
    /// If the branch does not exist.
    pub async fn open<P, S>(workspace_path: P, branch_name: S) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let branch_name = branch_name.as_ref();
        let branch_path = workspace_path.as_ref().join(branch_name);
        todo!("branch open")
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("branch already exists")]
    BranchAlreadyExists,
    #[error("unable to write branch `{path:?}`: `{message}`")]
    Io { path: PathBuf, message: String },
}
