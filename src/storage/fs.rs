use {
    super::{Error, StorageRead, StorageWrite},
    std::path::PathBuf,
    tokio::{fs, io::AsyncRead},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Fs {
    config: Config,
}
impl Fs {
    pub async fn new(config: Config) -> Result<Self, Error> {
        fs::create_dir_all(&config.path).await?;
        Ok(Self { config })
    }
}
#[async_trait::async_trait]
impl StorageWrite for Fs {
    async fn write<S, R>(&self, hash: S, r: R) -> Result<usize, Error>
    where
        S: AsRef<str> + Send,
        R: AsyncRead + Send,
    {
        todo!()
    }
}
