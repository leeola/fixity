use {
    crate::{core, Error},
    std::path::PathBuf,
};
const DEFAULT_FIXI_DIR_NAME: &str = ".fixi";
const DEFAULT_WORKSPACE_NAME: &str = "default";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub fixi_dir_path: PathBuf,
    pub workspace_name: String,
}
impl Config {
    pub fn builder() -> Builder {
        Builder::default()
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Builder {
    base_path: Option<PathBuf>,
    fixi_dir_name: Option<PathBuf>,
    workspace_name: Option<String>,
}
impl Builder {
    pub fn merge(&mut self, builder: Builder) {
        self.base_path(builder.base_path);
        self.fixi_dir_name(builder.fixi_dir_name);
        self.workspace_name(builder.workspace_name);
    }
    pub fn ensure_base_path<F>(&mut self, ensure_fn: F) -> &mut Self
    where
        F: FnOnce() -> PathBuf,
    {
        if self.base_path.is_none() {
            self.base_path = Some(ensure_fn());
        }
        self
    }
    pub fn ensure_fixi_dir_name<F>(&mut self, ensure_fn: F) -> &mut Self
    where
        F: FnOnce() -> PathBuf,
    {
        if self.fixi_dir_name.is_none() {
            self.fixi_dir_name = Some(ensure_fn());
        }
        self
    }
    pub fn base_path(&mut self, base_path: Option<PathBuf>) -> &mut Self {
        self.base_path = base_path;
        self
    }
    pub fn fixi_dir_name(&mut self, fixi_dir_name: Option<PathBuf>) -> &mut Self {
        self.fixi_dir_name = fixi_dir_name;
        self
    }
    pub fn workspace_name(&mut self, workspace_name: Option<String>) -> &mut Self {
        self.workspace_name = workspace_name;
        self
    }
    pub fn with_base_path(mut self, base_path: Option<PathBuf>) -> Self {
        self.base_path(base_path);
        self
    }
    pub fn with_fixi_dir_name(mut self, fixi_dir_name: Option<PathBuf>) -> Self {
        self.fixi_dir_name(fixi_dir_name);
        self
    }
    pub fn with_workspace_name(mut self, workspace_name: Option<String>) -> Self {
        self.workspace_name(workspace_name);
        self
    }
    fn seek_fixi_dir(&self) -> Result<PathBuf, Error> {
        let fixi_dir_name = self
            .fixi_dir_name
            .as_ref()
            .expect("expected default fixi_dir_name to be set prior to calling seek_fixi_dir");
        core::dir::resolve(fixi_dir_name, PathBuf::from(".")).ok_or(Error::RepositoryNotFound)
    }
    pub fn build(mut self) -> Result<Config, Error> {
        self.ensure_fixi_dir_name(|| PathBuf::from(DEFAULT_FIXI_DIR_NAME));
        let fixi_dir_name = self.fixi_dir_name.as_ref().expect("impossibly missing");
        let fixi_dir_path = match self.base_path {
            Some(base_path) => base_path.join(fixi_dir_name),
            None => self.seek_fixi_dir()?,
        };
        let workspace_name = self
            .workspace_name
            .unwrap_or_else(|| DEFAULT_WORKSPACE_NAME.to_owned());
        Ok(Config {
            fixi_dir_path,
            workspace_name,
        })
    }
}
