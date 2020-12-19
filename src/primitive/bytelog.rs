use {
    crate::{
        primitive::{appendlog::LogContainer, AppendLog, Flush},
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
pub struct ByteNode {
    pub chunks: Vec<Addr>,
}
pub struct ByteLog<'s, S> {
    log: AppendLog<'s, S>,
    _chunks: Vec<Addr>,
}
impl<'s, S> ByteLog<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        let log = AppendLog::new(storage, addr);
        Self {
            log,
            _chunks: Vec::new(),
        }
    }
}
impl<'s, S> ByteLog<'s, S>
where
    S: StorageRead,
{
    pub async fn first_container(&self) -> Result<Option<LogContainer<'_, ByteNode>>, Error> {
        let container = self.log.first_container::<ByteNode>().await?;
        Ok(container.map(|LogContainer { node, addr }| LogContainer {
            addr,
            node: node.inner,
        }))
    }
    pub async fn first(&self) -> Result<Option<ByteNode>, Error> {
        let container = self.first_container().await?;
        Ok(container.map(|LogContainer { node, .. }| node))
    }
}
pub trait ByteStash {
    fn put(&mut self, index: usize, bytes: &[u8]) -> Result<(), Error>;
    fn get(&mut self, index: usize) -> Result<Vec<u8>, Error>;
    fn drop(self) -> Result<(), Error>;
}
