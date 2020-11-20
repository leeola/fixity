use {
    crate::{fixity::Flush, Addr},
    std::{
        path::{Path, PathBuf},
        str::FromStr,
    },
    tokio::{
        fs::{self, File, OpenOptions},
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
/// A separator between the lhs and optional rhs of a [`Ref`].
const REF_SEP: &str = ": ";
const STAGE_SEP: &str = "\n";
/// The internal folder where branch HEADs are stored.
const BRANCHES_DIR: &str = "branches";
const REF_TYPE_ADDR: &str = "addr";
const REF_TYPE_REF: &str = "ref";
const HEAD_FILE_NAME: &str = "HEAD";
const INIT_HEAD_REF: &str = "refs/heads/default";
pub struct Head {
    workspace_path: PathBuf,
    state: State,
}
impl Head {
    /// Create a new `HEAD` at the specified [`Addr`].
    ///
    /// # Errors
    ///
    /// If the `HEAD` or default branch already exist.
    pub async fn init<P, S>(fixi_dir: P, workspace: S) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let workspace_path = fixi_dir.as_ref().join(workspace.as_ref());
        fs::create_dir(&workspace_path)
            .await
            .map_err(|source| Error::Init {
                path: workspace_path.clone(),
                message: format!("create workspace"),
            })?;
        fs::create_dir_all(workspace_path.join("refs").join("heads"))
            .await
            .map_err(|source| Error::Init {
                path: workspace_path.clone(),
                message: format!("create refs/heads"),
            })?;
        let state = State::Ref {
            ref_: INIT_HEAD_REF.to_owned(),
            addr: None,
            staged: None,
        };
        state.create(&workspace_path).await?;
        Ok(Self {
            workspace_path,
            state,
        })
    }
    /// Open an existing `HEAD`.
    pub async fn open<P, S>(fixi_dir: P, workspace: S) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let workspace_path = fixi_dir.as_ref().join(workspace.as_ref());
        let state = State::open(&workspace_path).await?;
        Ok(Self {
            workspace_path,
            state,
        })
    }
    /// Commit the address that `STAGE` is at to that of the branch the `HEAD` points to.
    pub async fn commit(&mut self) -> Result<Addr, Error> {
        match &mut self.state {
            State::Detached(_) => return Err(Error::DetatchedHead),
            State::Ref { ref_, addr, staged } => {
                {
                    let staged = match staged {
                        Some(staged) => staged,
                        None => return Err(Error::CommitEmptyStage),
                    };
                    let path = self.workspace_path.join(ref_);
                    let mut f = OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(&path)
                        .await
                        .map_err(|err| Error::Io {
                            path: path.to_owned(),
                            message: format!("open ref: {}", err),
                        })?;
                    f.write_all(staged.as_bytes())
                        .await
                        .map_err(|err| Error::Io {
                            path: path.to_owned(),
                            message: format!("write ref: {}", err),
                        })?;
                    f.sync_all().await.map_err(|err| Error::Io {
                        path: path.to_owned(),
                        message: format!("sync ref: {}", err),
                    })?;
                }
                let staged_addr = staged.take().expect("staged impossibly missing");
                addr.replace(staged_addr.clone());
                log::warn!("content hash being returned instead of commit hash");
                Ok(staged_addr)
            }
        }
    }
    /// Move the `HEAD`
    pub async fn stage(&mut self, addr: &Addr) -> Result<(), Error> {
        match &mut self.state {
            State::Detached(_) => return Err(Error::DetatchedHead),
            State::Ref { staged, .. } => {
                staged.replace(addr.clone());
            }
        }
        self.state.write(&self.workspace_path).await
    }
    pub fn addr(&self) -> Option<Addr> {
        match &self.state {
            State::Detached(addr) => Some(addr.clone()),
            State::Ref { addr, staged, .. } => staged.clone().or_else(|| addr.clone()),
        }
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
        let addr = self.inner.flush().await?;
        self.head.stage(&addr).await?;
        Ok(addr)
    }
    pub async fn commit(&mut self) -> Result<Addr, crate::Error> {
        let content_addr = self.inner.flush().await?;
        self.head.stage(&content_addr).await?;
        let addr = self.head.commit().await?;
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
/// An internal HEAD state, representing either a detatched HEAD or a pointer to a Ref,
/// with an optional stage value to move the ref to.
#[derive(Debug, Clone)]
enum State {
    Detached(Addr),
    Ref {
        ref_: String,
        /// An optional [`Addr`] that the `ref` resolved to.
        addr: Option<Addr>,
        /// An optional staged [`Addr`], to be
        staged: Option<Addr>,
    },
}
impl State {
    /// Open the given path and parse it into `State`.
    pub async fn open<P>(workspace_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let workspace_path = workspace_path.as_ref();
        let head_path = workspace_path.join(HEAD_FILE_NAME);
        let head_contents =
            match read_to_string(&head_path)
                .await
                .map_err(|err| Error::OpenRef {
                    path: head_path.to_owned(),
                    message: "failed to open HEAD".to_owned(),
                })? {
                Some(s) => s,
                None => {
                    return Err(Error::CorruptHead {
                        message: "HEAD is missing".to_owned(),
                    })
                }
            };
        if head_contents.is_empty() {
            return Err(Error::CorruptHead {
                message: "HEAD is empty".to_owned(),
            });
        }
        let (staged, head) = {
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
        let state = match (head_split.next(), head_split.next()) {
            (Some(REF_TYPE_ADDR), Some(addr)) => Self::Detached(addr.to_owned().into()),
            (Some(REF_TYPE_REF), Some(ref_)) => {
                let path = workspace_path.join(ref_);
                let addr = read_to_string(path.as_path())
                    .await
                    .map_err(|err| Error::OpenRef {
                        path: path.to_owned(),
                        message: "failed to open branch".to_owned(),
                    })?
                    .map(Addr::from);
                Self::Ref {
                    ref_: ref_.to_owned(),
                    addr,
                    staged,
                }
            }
            (Some(ref_type), _) => {
                return Err(Error::InvalidRef {
                    path: workspace_path.to_owned(),
                    message: format!("unrecognized ref type: {}", ref_type),
                })
            }
            (None, None) | (None, Some(_)) => unreachable!(
                "split has to return at least one item, and cannot return None then Some"
            ),
        };
        Ok(state)
    }
    pub async fn create<P>(&self, workspace_path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        self.write_or_create(workspace_path, true).await
    }
    pub async fn write<P>(&self, workspace_path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        self.write_or_create(workspace_path, false).await
    }
    async fn write_or_create<P>(&self, workspace_path: P, create_new: bool) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let path = workspace_path.as_ref().join(HEAD_FILE_NAME);
        let mut f = OpenOptions::new()
            .create_new(create_new)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
            .map_err(|err| Error::WriteRef {
                path: path.clone(),
                message: format!("create state: {}", err),
            })?;
        match self {
            Self::Detached(addr) => {
                f.write_all(addr.as_bytes())
                    .await
                    .map_err(|err| Error::WriteRef {
                        path: path.clone(),
                        message: format!("write state: {}", err),
                    })?;
            }
            Self::Ref {
                ref_,
                staged: Some(staged),
                ..
            } => {
                let body = format!("{}\nref: {}", staged.as_str(), ref_);
                f.write_all(body.as_bytes())
                    .await
                    .map_err(|err| Error::WriteRef {
                        path: path.clone(),
                        message: format!("write state: {}", err),
                    })?;
            }
            Self::Ref {
                ref_, staged: None, ..
            } => {
                let body = format!("ref: {}", ref_);
                f.write_all(body.as_bytes())
                    .await
                    .map_err(|err| Error::WriteRef {
                        path: path.to_owned(),
                        message: format!("write state: {}", err),
                    })?;
            }
        }
        f.sync_all().await.map_err(|err| Error::WriteRef {
            path: path.to_owned(),
            message: format!("syncing state: {}", err),
        })?;
        Ok(())
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
        Err(err) => return Err(dbg!(err)),
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
    #[error("cannot commit or stage on a detatched HEAD")]
    DetatchedHead,
    #[error("unable to init head `{path:?}`: `{message}`")]
    Init { path: PathBuf, message: String },
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
    #[error("io failure `{path:?}`: `{message}`")]
    Io { path: PathBuf, message: String },
}
