use {
    super::{Error, Guard, Init, Status, Workspace},
    crate::Addr,
    std::{
        ops::Drop,
        path::{Path, PathBuf},
        str::FromStr,
    },
    tokio::{
        fs::{self, File, OpenOptions},
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
const WORKSPACE_LOCK_FILE_NAME: &str = "WORKSPACE.lock";
const HEAD_FILE_NAME: &str = "HEAD";
const BRANCHES_DIR: &str = "branches";
/// A separator between the lhs and optional rhs of a [`Ref`].
const KV_SEP: &str = ": ";
const LINE_SEP: &str = "\n";
const DETACHED_KEY: &str = "detached";
const BRANCH_KEY: &str = "branch";
const STAGED_CONTENT_KEY: &str = "staged_content";
pub struct Config {
    pub workspaces_root_dir: PathBuf,
}
#[async_trait::async_trait]
impl Init for Config {
    type Workspace = Fs;
    async fn init(&self, workspace: String) -> Result<Self::Workspace, Error> {
        Fs::init(self.workspaces_root_dir.clone(), workspace).await
    }
    async fn open(&self, workspace: String) -> Result<Self::Workspace, Error> {
        Fs::open(self.workspaces_root_dir.clone(), workspace).await
    }
}
pub struct Fs {
    workspaces_root_dir: PathBuf,
    workspace: String,
}
impl Fs {
    pub async fn init(workspaces_root_dir: PathBuf, workspace: String) -> Result<Self, Error> {
        fs::create_dir(&workspaces_root_dir)
            .await
            .map_err(|source| Error::Internal(format!("create workspaces dir: {}", source)))?;
        let workspace_path = workspaces_root_dir.join(&workspace);
        fs::create_dir(&workspace_path)
            .await
            .map_err(|source| Error::Internal(format!("create workspace dir: {}", source)))?;
        fs::create_dir_all(workspace_path.join("branches"))
            .await
            .map_err(|source| Error::Internal(format!("create branches dir: {}", source)))?;
        let head_path = workspaces_root_dir.join(&workspace).join(HEAD_FILE_NAME);
        HeadState::Branch {
            branch: "default".to_owned(),
            staged_content: None,
        }
        .write(head_path.as_path(), true)
        .await?;
        Ok(Self {
            workspaces_root_dir,
            workspace,
        })
    }
    pub async fn open(workspaces_root_dir: PathBuf, workspace: String) -> Result<Self, Error> {
        let _ = HeadState::open(
            workspaces_root_dir
                .join(&workspace)
                .join(HEAD_FILE_NAME)
                .as_path(),
        )
        .await?;
        Ok(Self {
            workspaces_root_dir,
            workspace,
        })
    }
}
async fn fetch_branch_addr<P: AsRef<Path>>(branch_path: P) -> Result<Option<Addr>, Error> {
    let branch_path = branch_path.as_ref();
    let branch_contents = read_to_string(branch_path)
        .await
        .map_err(|err| Error::Internal(format!("open BRANCH `{:?}`, `{}`", branch_path, err)))?;
    branch_contents
        .map(|addr| {
            Addr::decode(addr).ok_or_else(|| Error::Internal("HEAD branch invalid Addr".to_owned()))
        })
        .transpose()
}
#[async_trait::async_trait]
impl Workspace for Fs {
    type Guard<'a> = FsGuard<'a>;
    async fn lock(&self) -> Result<Self::Guard<'_>, Error> {
        let file_lock_path = self
            .workspaces_root_dir
            .join(&self.workspace)
            .join(WORKSPACE_LOCK_FILE_NAME);
        // using a non-async File since we're going to drop it in a blocking manner.
        let file_lock_res = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&file_lock_path);
        let workspace_guard_file = match file_lock_res {
            Ok(f) => f,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(Error::InUse)
            },
            Err(err) => {
                return Err(Error::Internal(format!(
                    "failed to acquire workspace lock: {}",
                    err
                )))
            },
        };
        Ok(FsGuard {
            workspaces_root_dir: &self.workspaces_root_dir.as_path(),
            workspace: self.workspace.as_str(),
            workspace_guard_file: Some(workspace_guard_file),
            file_lock_path,
        })
    }
    async fn status(&self) -> Result<Status, Error> {
        status(self.workspaces_root_dir.as_path(), self.workspace.as_ref()).await
    }
}
async fn status(workspaces_root_dir: &Path, workspace: &str) -> Result<Status, Error> {
    let head_path = workspaces_root_dir.join(workspace).join(HEAD_FILE_NAME);
    let head_state = HeadState::open(head_path.as_path()).await?;
    let status = match head_state {
        HeadState::Detached { addr } => Status::Detached(addr),
        HeadState::Branch {
            branch,
            staged_content,
        } => {
            let branch_path = workspaces_root_dir
                .join(workspace)
                .join(BRANCHES_DIR)
                .join(&branch);
            let branch_addr = fetch_branch_addr(branch_path).await?;
            match (staged_content, branch_addr) {
                (None, None) => Status::Init { branch },
                (Some(staged_content), None) => Status::InitStaged {
                    branch,
                    staged_content,
                },
                (None, Some(addr)) => Status::Clean {
                    branch,
                    commit: addr,
                },
                (Some(staged_content), Some(addr)) => Status::Staged {
                    branch,
                    staged_content,
                    commit: addr,
                },
            }
        },
    };
    Ok(status)
}
// allowing name repetition since this is a Guard for the Fs type. Seems logical.
#[allow(clippy::module_name_repetitions)]
pub struct FsGuard<'a> {
    workspaces_root_dir: &'a Path,
    workspace: &'a str,
    workspace_guard_file: Option<std::fs::File>,
    file_lock_path: PathBuf,
}
#[async_trait::async_trait]
impl<'a> Guard for FsGuard<'a> {
    async fn status(&self) -> Result<Status, Error> {
        status(self.workspaces_root_dir, self.workspace).await
    }
    async fn stage(&self, stage_addr: Addr) -> Result<(), Error> {
        let head_path = self
            .workspaces_root_dir
            .join(self.workspace)
            .join(HEAD_FILE_NAME);
        let head_state = HeadState::open(&head_path).await?;
        let new_state = match head_state {
            HeadState::Detached { .. } => return Err(Error::DetachedHead),
            HeadState::Branch { branch, .. } => HeadState::Branch {
                branch,
                staged_content: Some(stage_addr),
            },
        };
        new_state.write(head_path.as_path(), false).await?;
        Ok(())
    }
    async fn commit(&self, commit_addr: Addr) -> Result<(), Error> {
        let head_path = self
            .workspaces_root_dir
            .join(self.workspace)
            .join(HEAD_FILE_NAME);
        let head_state = HeadState::open(&head_path).await?;
        let branch = match head_state {
            HeadState::Detached { .. } => return Err(Error::DetachedHead),
            HeadState::Branch {
                staged_content: None,
                ..
            } => {
                return Err(Error::CommitEmptyStage);
            },
            HeadState::Branch {
                branch,
                staged_content: Some(_),
            } => branch,
        };
        {
            let branch_path = self
                .workspaces_root_dir
                .join(self.workspace)
                .join(BRANCHES_DIR)
                .join(&branch);
            let mut f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(branch_path)
                .await
                .map_err(|err| Error::Internal(format!("open branch: {}", err)))?;
            f.write_all(&commit_addr.long().into_bytes())
                .await
                .map_err(|err| Error::Internal(format!("write branch to HEAD: {}", err)))?;
            f.sync_all()
                .await
                .map_err(|err| Error::Internal(format!("sync branch to HEAD: {}", err)))?;
        }
        HeadState::Branch {
            branch,
            staged_content: None,
        }
        .write(head_path.as_path(), false)
        .await?;
        Ok(())
    }
}
impl<'a> Drop for FsGuard<'a> {
    fn drop(&mut self) {
        let _ = self.workspace_guard_file.take();
        if let Err(err) = std::fs::remove_file(&self.file_lock_path) {
            log::info!(
                "failed to release workspace lock. path:{:?}, err:{}",
                self.file_lock_path,
                err,
            );
        }
    }
}
#[derive(Debug)]
enum HeadState {
    Detached {
        addr: Addr,
    },
    Branch {
        branch: String,
        staged_content: Option<Addr>,
    },
    /* Workspace {
     *     workspace: String,
     *     branch: String,
     * },
     * Remote {
     *     remote: String,
     *     workspace: String,
     *     branch: String,
     * },
     */
}
impl HeadState {
    pub async fn open(head_path: &Path) -> Result<Self, Error> {
        let head_contents = read_to_string(&head_path)
            .await
            .map_err(|err| {
                Error::Internal(format!("open HEAD for read `{:?}`, `{}`", head_path, err))
            })?
            .ok_or_else(|| Error::Internal(format!("missing HEAD at `{:?}`", head_path)))?;
        head_contents.parse()
    }
    pub async fn write(self, head_path: &Path, create_new: bool) -> Result<(), Error> {
        let bytes = self.format().into_bytes();
        let mut f = OpenOptions::new()
            .create_new(create_new)
            .truncate(true)
            .write(true)
            .open(head_path)
            .await
            .map_err(|err| Error::Internal(format!("open HEAD for writing: {}", err)))?;
        f.write_all(&bytes)
            .await
            .map_err(|err| Error::Internal(format!("write HEAD: {}", err)))?;
        f.sync_all()
            .await
            .map_err(|err| Error::Internal(format!("sync HEAD: {}", err)))?;
        Ok(())
    }
    pub fn format(self) -> String {
        match self {
            HeadState::Detached { addr } => format!("{}{}{}", DETACHED_KEY, KV_SEP, addr.long()),
            HeadState::Branch {
                branch,
                staged_content,
            } => {
                if let Some(staged_content) = staged_content {
                    format!(
                        "{}{}{}\n{}{}{}",
                        BRANCH_KEY,
                        KV_SEP,
                        branch,
                        STAGED_CONTENT_KEY,
                        KV_SEP,
                        staged_content.long()
                    )
                } else {
                    format!("{}{}{}", BRANCH_KEY, KV_SEP, branch)
                }
            },
        }
    }
}
impl FromStr for HeadState {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.split(LINE_SEP);
        let mut head_line = lines
            .next()
            .expect("first value of split impossibly missing")
            .splitn(2, KV_SEP);
        let state_key = head_line
            .next()
            .expect("first value of split impossibly missing");
        let head_state = match state_key {
            DETACHED_KEY => {
                let addr = head_line
                    .next()
                    .ok_or_else(|| Error::Internal("detached value missing".to_owned()))?;
                Self::Detached {
                    addr: Addr::decode(addr)
                        .ok_or_else(|| Error::Internal("detached HEAD invalid Addr".to_owned()))?,
                }
            },
            BRANCH_KEY => {
                let branch = head_line
                    .next()
                    .ok_or_else(|| Error::Internal("branch value missing".to_owned()))?
                    .to_owned();
                let staged_content = lines
                    .next()
                    .map(|staged_line| {
                        let mut staged_line = staged_line.splitn(2, KV_SEP);
                        let staged_key = staged_line
                            .next()
                            .expect("first value of split impossibly missing");
                        if staged_key != STAGED_CONTENT_KEY {
                            return Err(Error::Internal(format!(
                                "unknown HEAD staged_content key `{:?}`",
                                staged_key
                            )));
                        }
                        let addr = staged_line.next().ok_or_else(|| {
                            Error::Internal("staged_content value missing".to_owned())
                        })?;
                        Addr::decode(addr).ok_or_else(|| {
                            Error::Internal("HEAD staged_content invalid Addr".to_owned())
                        })
                    })
                    .transpose()?;
                Self::Branch {
                    branch,
                    staged_content,
                }
            },
            state_key => {
                return Err(Error::Internal(format!(
                    "unknown HEAD state `{:?}`",
                    state_key
                )));
            },
        };
        Ok(head_state)
    }
}
/// A helper to abstract the file opening behavior.
async fn read_to_string(path: &Path) -> Result<Option<String>, std::io::Error> {
    let mut s = String::new();
    let mut f = match File::open(path).await {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        },
        Err(err) => return Err(err),
    };
    f.read_to_string(&mut s).await?;
    Ok(Some(s))
}
