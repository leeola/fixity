use {
    super::{Error, StorageRead, StorageWrite},
    std::path::PathBuf,
    tokio::{
        fs::{self, OpenOptions},
        io::{self, AsyncRead},
    },
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
    async fn write<S, R>(&self, hash: S, mut r: R) -> Result<u64, Error>
    where
        S: AsRef<str> + 'static + Send,
        R: AsyncRead + Unpin + Send,
    {
        let hash = hash.as_ref();
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.config.path.join(hash))
            .await?;
        let n = io::copy(&mut r, &mut f).await?;
        Ok(n)
    }
}
