use {
    super::{Error, StorageRead, StorageWrite},
    crate::Addr,
    std::path::PathBuf,
    tokio::{
        fs::{self, OpenOptions},
        io::{self, AsyncRead, AsyncWrite},
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
    pub async fn init(config: Config) -> Result<Self, Error> {
        fs::create_dir(&config.path).await?;
        Ok(Self { config })
    }
    pub fn open(config: Config) -> Result<Self, Error> {
        Ok(Self { config })
    }
    fn resolve_path<A: AsRef<Addr>>(&self, addr: A) -> PathBuf {
        let addr = addr.as_ref();
        // TODO: encode as base16 on OSX, due to non-case sensitive FS.. or at least
        // as a storage feature? Or maybe runtime feature? Hmm.
        let file_name = multibase::encode(multibase::Base::Base58Btc, addr.as_bytes());
        // TODO: split the early bits of the path as subfolders. Reducing number of files
        // in a single dir.
        self.config.path.join(file_name)
    }
}
#[async_trait::async_trait]
impl StorageRead for Fs {
    async fn read<A, W>(&self, addr: A, mut w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let path = self.resolve_path(addr);
        let mut f = OpenOptions::new().read(true).open(path).await?;
        let n = io::copy(&mut f, &mut w).await?;
        Ok(n)
    }
}
#[async_trait::async_trait]
impl StorageWrite for Fs {
    async fn write<A, R>(&self, addr: A, mut r: R) -> Result<u64, Error>
    where
        A: AsRef<Addr> + 'static + Send,
        R: AsyncRead + Unpin + Send,
    {
        let path = self.resolve_path(addr);
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .await?;
        let n = io::copy(&mut r, &mut f).await?;
        Ok(n)
    }
}
