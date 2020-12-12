use {
    crate::{
        primitive::{AppendLog, Build, Flush, GetAddr, InsertAddr},
        storage::{StorageRead, StorageWrite},
        Addr, Error,
    },
    chrono::{DateTime, Utc},
};

// pub type CommitLog<'s, S> = AppendLog<'s, S, Commit>;

#[derive(Debug)]
pub struct Commit {
    pub date: DateTime<Utc>,
    pub addr: Addr,
}

pub struct CommitLog<'s, S, Inner> {
    log: AppendLog<'s, S>,
    inner: Inner,
}
impl<'s, S, Inner> CommitLog<'s, S, Inner> {
    pub fn new(storage: &'s S, addr: Option<Addr>, inner: Inner) -> Self {
        let log = AppendLog::new(storage, addr);
        Self { log, inner }
    }
}
#[async_trait::async_trait]
impl<'s, S, Inner> Flush for CommitLog<'s, S, Inner>
where
    S: StorageRead + StorageWrite,
    Inner: Flush + Sync + Send,
{
    async fn flush(&mut self) -> Result<Addr, Error> {
        let addr = self.inner.flush().await?;
        self.log
            .append(Commit {
                date: Utc::now(),
                addr,
            })
            .await
    }
}
