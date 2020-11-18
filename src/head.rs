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
/// A separator between the lhs and optional rhs of a [`Ref`].
const REF_SEP: &str = ": ";
const STAGE_SEP: &str = "\n";
/// The internal folder where branch HEADs are stored.
const BRANCHES_DIR: &str = "branches";
const REF_TYPE_ADDR: &str = "addr";
const REF_TYPE_BRANCH: &str = "branch";
const REF_TYPE_HEAD: &str = "head";
const HEAD_FILE_NAME: &str = "HEAD";
pub struct Head {
    workspace_path: PathBuf,
    head: Option<Ref>,
}
impl Head {
    /// Create a new `HEAD` at the specified [`Addr`].
    ///
    /// # Errors
    ///
    /// If the `HEAD` or default branch already exist.
    pub async fn init<P, S>(fixi_dir: P, workspace: S, addr: Addr) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let workspace_path = fixi_dir.as_ref().join(workspace.as_ref());
        let head_path = workspace_path.join(HEAD_FILE_NAME);
        let mut f = OpenOptions::new()
            .create_new(true)
            .open(&head_path)
            .await
            .map_err(|err| Error::OpenRef {
                path: head_path.clone(),
                message: format!("create head: {}", err),
            })?;
        f.sync_all().await.map_err(|err| Error::OpenRef {
            path: head_path.clone(),
            message: format!("syncing head: {}", err),
        })?;
        Ok(Self {
            workspace_path,
            head: None,
        })
    }
    /// Open an existing `HEAD`.
    pub async fn open<P, S>(fixi_dir: P, workspace: S) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let workspace_path = fixi_dir.as_ref().join(workspace.as_ref());
        let head_path = workspace_path.join(HEAD_FILE_NAME);
        let head = Ref::open(head_path).await?;
        Ok(Self {
            workspace_path,
            head,
        })
    }
    /// Commit the address that `STAGE` is at to that of the branch the `HEAD` points to.
    pub async fn commit(&mut self) -> Result<(), Error> {
        // match self.head
        // let stage_addr = stage.addr();
        // if inner.head.addr() == stage_addr {
        //     return Ok(());
        // }
        // // let branch_name = match inner.head {
        // //     Ref::Addr(_) => return Err(Error::DetatchedHead),
        // //     Ref::Branch { branch, .. } => branch,
        // // };
        todo!("move branch")
    }
    /// Move the `HEAD`
    pub async fn stage(&mut self, addr: &Addr) -> Result<(), Error> {
        if let Some(inner) = self.inner.as_ref() {
            if addr == inner.stage.addr() {
                return Ok(());
            }
        }
        let stage_ref = StageRef::Addr(addr.clone());
        stage_ref.write(&self.workspace_path).await
    }
    /// Return the address `STAGE` points to.
    pub fn addr(&self) -> Option<Addr> {
        self.inner
            .as_ref()
            .map(|InnerHead { stage, .. }| stage.addr().clone())
    }
}
pub struct Guard<T> {
    head: Head,
    inner: T,
}
impl<T> Guard<T> {
    pub fn new(head: Head, inner: T) -> Self {
        Self { head, inner }
    }
}
impl<T> Guard<T>
where
    T: Flush,
{
    pub async fn stage(&mut self) -> Result<Addr, crate::Error> {
        todo!("guard stage")
    }
    pub async fn commit(&mut self) -> Result<Addr, crate::Error> {
        let addr = self.inner.flush().await?;
        self.head.stage(&addr).await?;
        self.head.commit().await?;
        Ok(addr)
    }
}
impl<T> std::ops::Deref for Guard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> std::ops::DerefMut for Guard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
#[derive(Debug, Clone)]
pub enum Ref {
    Addr(Addr),
    BranchWithStaged {
        branch: String,
        staged: Addr,
        addr: Addr,
    },
    Branch {
        branch: String,
        addr: Addr,
    },
}
impl Ref {
    /// Open the given path as a [HEAD Ref](Ref), with a `None` if the file does not exist.
    pub async fn open<P>(path: P) -> Result<Option<Self>, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let head_contents = read_to_string(path).await.map_or_else(
            |err| {
                Err(Error::OpenRef {
                    path: path.to_owned(),
                    message: "failed to open HEAD".to_owned(),
                })
            },
            |s| match s {
                Some(s) => Ok(s),
                None => Err(Error::DanglingHead),
            },
        )?;
        if head_contents.is_empty() {
            return Ok(None);
        }
        let (staged_addr, head) = {
            let mut maybe_staged = head_contents.splitn(2, STAGE_SEP);
            match (maybe_staged.next(), maybe_staged.next()) {
                (Some(stage_addr), Some(head)) => (Some(Addr::from(stage_addr.clone())), head),
                (Some(head), None) => (None, head),
                (None, None) | (None, Some(_)) => unreachable!(
                    "split has to return at least one item, and cannot return None then Some"
                ),
            }
        };
        let mut head_split = head.splitn(2, REF_SEP);
        let ref_ = match (head_split.next(), head_split.next()) {
            (Some(REF_TYPE_ADDR), Some(addr)) => Self::Addr(addr.to_owned().into()),
            (Some(REF_TYPE_BRANCH), Some(branch_name)) => {
                let path = path.join(BRANCHES_DIR).join(branch_name);
                let branch_addr = read_to_string(path.as_path()).await.map_or_else(
                    |err| {
                        Err(Error::OpenRef {
                            path: path.to_owned(),
                            message: "failed to open branch".to_owned(),
                        })
                    },
                    |s| match s {
                        Some(branch_addr) => Ok(Addr::from(branch_addr)),
                        None => Err(Error::DanglingHead),
                    },
                )?;
                Self::Branch {
                    branch: branch_name.to_owned(),
                    addr: branch_addr.into(),
                }
            }
            (Some(ref_type), _) => {
                return Err(Error::InvalidRef {
                    path: path.to_owned(),
                    message: format!("unrecognized ref type: {}", ref_type),
                })
            }
            (None, None) | (None, Some(_)) => unreachable!(
                "split has to return at least one item, and cannot return None then Some"
            ),
        };
        Ok(Some(ref_))
    }
    /// Return the underlying addr for this `Ref`.
    pub fn addr(&self) -> &Addr {
        match self {
            Ref::Addr(addr) | Ref::Branch { addr, .. } => &addr,
        }
    }
}
/// A helper to abstract the file opening behavior.
async fn read_to_string(path: &Path) -> Result<Option<String>, std::io::Error> {
    let mut s = String::new();
    let mut f = match File::open(path).await {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(err) => return Err(err),
    };
    f.read_to_string(&mut s).await?;
    Ok(Some(s))
}
async fn write_string_to_path<P>(path: P, s: String) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    let mut f = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(path.as_ref())
        .await?;
    f.write_all(s.as_bytes()).await?;
    f.sync_all().await?;
    Ok(())
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot commit empty STAGE")]
    CommitEmptyStage,
    #[error("cannot commit detatched HEAD")]
    DetatchedHead,
    #[error("unable to open ref `{path:?}`: `{message}`")]
    OpenRef { path: PathBuf, message: String },
    #[error("unable to write ref `{path:?}`: `{message}`")]
    WriteRef { path: PathBuf, message: String },
    #[error("invalid ref `{path:?}`: `{message}`")]
    InvalidRef { path: PathBuf, message: String },
    #[error("corrupt head: `{message}`")]
    CorruptHead { message: String },
    #[error("corrupt branch `{branch}`: `{message}`")]
    CorruptBranch { branch: String, message: String },
    #[error("HEAD pointing to ref or commit that's not found")]
    DanglingHead,
}
