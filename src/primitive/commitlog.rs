use {
    crate::{
        primitive::{appendlog::LogContainer, AppendLog},
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
}
impl<'s, S> CommitLog<'s, S>
where
    S: StorageRead,
{
    pub async fn first_container(&self) -> Result<Option<LogContainer<'_, CommitNode>>, Error> {
        let container = self.log.first_container::<CommitNode>().await?;
        Ok(container.map(|LogContainer { node, addr }| LogContainer {
            addr,
            node: node.inner,
        }))
    }
    pub async fn first(&self) -> Result<Option<CommitNode>, Error> {
        let container = self.first_container().await?;
        Ok(container.map(|LogContainer { node, .. }| node))
    }
}
impl<'s, S> CommitLog<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn append(&mut self, content: Addr) -> Result<Addr, Error> {
        let container = self.first_container().await?;
        if let Some(LogContainer {
            addr: old_addr,
            node:
                CommitNode {
                    content: old_content,
                    ..
                },
        }) = container
        {
            if content == old_content {
                return Ok(old_addr.clone());
            }
        }
        self.log
            .append(CommitNode {
                timestamp: Utc::now().timestamp(),
                content,
            })
            .await
    }
}
