use {
    crate::Addr,
    std::{
        path::{Path, PathBuf},
        str::FromStr,
    },
    tokio::{
        fs::{File, OpenOptions},
        io::AsyncReadExt,
    },
};
/// A separator between the lhs and optional rhs of a [`Ref`].
const REF_SEP: &str = ": ";
/// The internal folder where branch HEADs are stored.
const BRANCHES_DIR: &str = "branches";
const REF_TYPE_ADDR: &str = "addr";
const REF_TYPE_BRANCH: &str = "branch";
const REF_TYPE_HEAD: &str = "head";
const HEAD_FILE_NAME: &str = "HEAD";
const STAGE_FILE_NAME: &str = "STAGE";
pub struct Head {
    inner: Option<InnerHead>,
}
struct InnerHead {
    stage: StageRef,
    head: Ref,
}
impl Head {
    /// Create a new `HEAD` at the specified [`Addr`].
    ///
    /// # Errors
    ///
    /// If the `HEAD`, `STAGE` or default branch already exist.
    pub async fn init<P, S>(fixi_dir: P, workspace: S, addr: Addr) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        todo!("head init")
    }
    /// Open an existing `HEAD`.
    pub async fn open<P, S>(fixi_dir: P, workspace: S) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let head_ref = {
            let path = fixi_dir
                .as_ref()
                .join(workspace.as_ref())
                .join(HEAD_FILE_NAME);
            Ref::open(path).await?
        };
        let stage_ref = {
            let path = fixi_dir
                .as_ref()
                .join(workspace.as_ref())
                .join(STAGE_FILE_NAME);
            StageRef::open(path).await?
        };
        match (head_ref, stage_ref) {
            (Some(head), Some(stage)) => Ok(Self {
                inner: Some(InnerHead { stage, head }),
            }),
            (None, None) => Ok(Self { inner: None }),
            (Some(_), None) => Err(Error::CorruptHead {
                message: "have HEAD, missing STAGE".to_owned(),
            }),
            (None, Some(_)) => Err(Error::CorruptHead {
                message: "missing HEAD, have STAGE".to_owned(),
            }),
        }
    }
    /// Return the address HEAD points to.
    pub fn addr(&self) -> Option<Addr> {
        todo!("head addr")
    }
    /// Return the address HEAD points to.
    pub fn stage_addr(&self) -> Option<Addr> {
        todo!("head addr")
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
    pub async fn stage(&self) -> Result<Addr, crate::Error> {
        todo!("guard stage")
    }
    pub async fn commit(&self) -> Result<Addr, crate::Error> {
        todo!("guard commit")
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
pub enum Ref {
    Addr(Addr),
    Branch { branch: String, addr: Addr },
}
impl Ref {
    /// Open the given path as a [HEAD Ref](Ref), with a `None` if the file does not exist.
    pub async fn open<P>(path: P) -> Result<Option<Self>, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let head_ref = match open_ref_path(path).await? {
            Some(branch_addr) => branch_addr,
            None => return Ok(None),
        };
        let mut split = head_ref.splitn(2, REF_SEP);
        let ref_ = match (split.next(), split.next()) {
            (Some(REF_TYPE_ADDR), Some(addr)) => Self::Addr(addr.to_owned().into()),
            (Some(REF_TYPE_BRANCH), Some(branch_name)) => {
                let path = path.join(BRANCHES_DIR).join(branch_name);
                let branch_addr = match open_ref_path(path.as_path()).await? {
                    Some(branch_addr) => branch_addr,
                    None => return Ok(None),
                };
                Self::Branch {
                    branch: branch_name.to_owned(),
                    addr: branch_addr.into(),
                }
            }
            (_, _) => {
                return Err(Error::InvalidRef {
                    path: path.to_owned(),
                    message: "unexpected ref body".to_owned(),
                })
            }
        };
        Ok(Some(ref_))
    }
}
/// A helper to abstract the ref opening behavior over [`Ref::open`] and
/// [`StageRef::open`].
async fn open_ref_path(path: &Path) -> Result<Option<String>, Error> {
    let mut s = String::new();
    let mut f = match File::open(path).await {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(err) => {
            return Err(Error::OpenRef {
                path: path.to_owned(),
                message: err.to_string(),
            })
        }
    };
    f.read_to_string(&mut s)
        .await
        .map_err(|err| Error::OpenRef {
            path: path.to_owned(),
            message: err.to_string(),
        })?;
    Ok(Some(s))
}
pub enum StageRef {
    Head(Ref),
    Addr(Addr),
    Branch { branch: String, addr: Addr },
}
impl StageRef {
    /// Open the given path as a [HEAD Ref](Ref), with a `None` if the file does not exist.
    pub async fn open<P>(path: P) -> Result<Option<Self>, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let head_ref = match open_ref_path(path).await? {
            Some(branch_addr) => branch_addr,
            None => return Ok(None),
        };
        let mut split = head_ref.splitn(2, REF_SEP);
        let ref_ = match (split.next(), split.next()) {
            (Some(REF_TYPE_ADDR), Some(addr)) => Self::Addr(addr.to_owned().into()),
            (Some(REF_TYPE_BRANCH), Some(branch_name)) => {
                let path = path.join(BRANCHES_DIR).join(branch_name);
                let branch_addr = match open_ref_path(path.as_path()).await? {
                    Some(branch_addr) => branch_addr,
                    None => {
                        return Err(Error::CorruptBranch {
                            branch: branch_name.to_owned(),
                            message: "ref points to branch that does not exist".to_owned(),
                        })
                    }
                };
                Self::Branch {
                    branch: branch_name.to_owned(),
                    addr: branch_addr.into(),
                }
            }
            (Some(REF_TYPE_HEAD), None) => {
                let head_ref = match Ref::open(path).await.map_err(|err| Error::OpenRef {
                    path: path.to_owned(),
                    message: err.to_string(),
                })? {
                    Some(head_ref) => head_ref,
                    None => return Ok(None),
                };
                head_ref.into()
            }
            (_, _) => {
                return Err(Error::InvalidRef {
                    path: path.to_owned(),
                    message: "unexpected ref body".to_owned(),
                })
            }
        };
        Ok(Some(ref_))
    }
}
impl From<Ref> for StageRef {
    fn from(ref_: Ref) -> Self {
        match ref_ {
            Ref::Addr(addr) => Self::Addr(addr),
            Ref::Branch { branch, addr } => Self::Branch { branch, addr },
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to open ref `{path:?}`: `{message}`")]
    OpenRef { path: PathBuf, message: String },
    #[error("invalid ref `{path:?}`: `{message}`")]
    InvalidRef { path: PathBuf, message: String },
    #[error("corrupt head: `{message}`")]
    CorruptHead { message: String },
    #[error("corrupt branch `{branch}`: `{message}`")]
    CorruptBranch { branch: String, message: String },
}
