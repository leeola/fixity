use {
    crate::{
        primitive::{AppendLog, Flush},
        storage::{StorageRead, StorageWrite},
        Addr, Error,
    },
    chrono::Utc,
};
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug)]
pub struct CommitNode {
    pub timestamp: i64,
    pub content: Addr,
}
pub struct CommitLog<'s, S> {
    log: AppendLog<'s, S>,
}
impl<'s, S> CommitLog<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        let log = AppendLog::new(storage, addr);
        Self { log }
    }
    pub fn wrap_inner<Inner>(self, inner: Inner) -> Commit<'s, S, Inner> {
        let Self { log } = self;
        Commit { log, inner }
    }
}
impl<'s, S> CommitLog<'s, S>
where
    S: StorageRead,
{
    pub async fn first(&self) -> Result<Option<CommitNode>, Error> {
        let log_node = self.log.first::<CommitNode>().await?;
        Ok(log_node.map(|log_node| log_node.inner))
    }
}
pub struct Commit<'s, S, Inner> {
    log: AppendLog<'s, S>,
    inner: Inner,
}
#[async_trait::async_trait]
impl<'s, S, Inner> Flush for Commit<'s, S, Inner>
where
    S: StorageRead + StorageWrite,
    Inner: Flush + Sync + Send,
{
    async fn flush(&mut self) -> Result<Addr, Error> {
        let content = self.inner.flush().await?;
        self.log
            .append(CommitNode {
                timestamp: Utc::now().timestamp(),
                content,
            })
            .await
    }
}
impl<'s, S, Inner> std::ops::Deref for Commit<'s, S, Inner> {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'s, S, Inner> std::ops::DerefMut for Commit<'s, S, Inner> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
