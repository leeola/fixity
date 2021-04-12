use {
    crate::{
        config::{Builder as ConfigBuilder, Config},
        core::{
            self,
            cache::{CacheRead, CacheWrite, DeserCache},
            fixity::InitError,
            storage,
            workspace::{self, Workspace},
        },
        Bytes, Error, Map, Path,
    },
    tokio::fs,
};
const STORAGE_DIR: &str = "storage";
const WORKSPACES_DIR: &str = "workspaces";
pub struct Fixity(Box<dyn InnerFixity>);
impl Fixity {
    pub fn builder() -> Builder {
        Default::default()
    }
    pub fn memory() -> Self {
        Self(Box::new(core::Fixity::memory()))
    }
    pub fn bytes(&self, path: Path) -> Result<Bytes<'_>, Error> {
        self.0.inner_bytes(path)
    }
    pub fn map(&self, path: Path) -> Map<'_> {
        self.0.inner_map(path)
    }
}
trait InnerFixity {
    fn inner_bytes(&self, path: Path) -> Result<Bytes<'_>, Error>;
    fn inner_map(&self, path: Path) -> Map<'_>;
}
impl<C, W> InnerFixity for core::Fixity<C, W>
where
    C: CacheRead + CacheWrite,
    W: Workspace,
{
    fn inner_bytes(&self, path: Path) -> Result<Bytes<'_>, Error> {
        let b = self.bytes(path)?;
        Ok(Bytes::new::<C, W>(b))
    }
    fn inner_map(&self, path: Path) -> Map<'_> {
        let m = self.map(path);
        Map::new::<C, W>(m)
    }
}
#[derive(Default)]
pub struct Builder {
    config: ConfigBuilder,
}
impl Builder {
    pub fn with_config(mut self, config: ConfigBuilder) -> Self {
        self.config.merge(config);
        self
    }
    pub async fn init(self) -> Result<Fixity, Error> {
        let (f, _) = self.init_config().await?;
        Ok(f)
    }
    pub async fn init_config(mut self) -> Result<(Fixity, Config), Error> {
        use std::path::PathBuf;
        // Setting a default "." base path if it's not already specified means the config
        // won't have to find a fixi dir. Ie we're creating a new repo, default new
        // location to current dir.
        self.config.ensure_base_path(|| PathBuf::from("."));
        let config = self.config.build()?;
        fs::create_dir(&config.fixi_dir_path)
            .await
            .map_err(|source| InitError::CreateDir { source })?;
        let storage = storage::Fs::init(storage::fs::Config {
            path: config.fixi_dir_path.join(STORAGE_DIR),
        })
        .await
        .map_err(|source| InitError::Storage { source })?;
        // init the Workspace
        let workspace = workspace::Fs::init(
            config.fixi_dir_path.join(WORKSPACES_DIR),
            config.workspace_name.clone(),
        )
        .await?;
        let fixity = Fixity(Box::new(core::Fixity::new(
            DeserCache::new(storage),
            workspace,
        )));
        Ok((fixity, config))
    }
    pub async fn open(self) -> Result<Fixity, Error> {
        let config = self.config.build()?;
        let storage = storage::Fs::open(storage::fs::Config {
            path: config.fixi_dir_path.join(STORAGE_DIR),
        })
        .map_err(|source| InitError::Storage { source })?;
        // open the Workspace
        let workspace = workspace::Fs::open(
            config.fixi_dir_path.join(WORKSPACES_DIR),
            config.workspace_name,
        )
        .await?;
        Ok(Fixity(Box::new(core::Fixity::new(
            DeserCache::new(storage),
            workspace,
        ))))
    }
}
