use crate::{
    deser::{Deser, Deserialize, Serialize},
    storage::{StorageRead, StorageWrite},
    Addr, Error,
};
pub struct LogContainer<'a, T> {
    pub addr: &'a Addr,
    pub node: T,
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct LogNode<T> {
    pub inner: T,
    pub prev: Option<Addr>,
}
pub struct AppendLog<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
}
impl<'s, S> AppendLog<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
}
impl<'s, S> AppendLog<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn append<T>(&mut self, inner: T) -> Result<Addr, Error>
    where
        T: Serialize + Deserialize,
    {
        let buf = {
            let node = LogNode {
                inner,
                prev: self.addr.clone(),
            };
            Deser::default().serialize(&node)?
        };
        let addr = Addr::hash(&buf);
        self.storage.write(addr.clone(), &*buf).await?;
        let _ = self.addr.replace(addr.clone());
        Ok(addr)
    }
}
impl<'s, S> AppendLog<'s, S>
where
    S: StorageRead,
{
    pub async fn first_container<T>(&self) -> Result<Option<LogContainer<'_, LogNode<T>>>, Error>
    where
        T: Deserialize,
    {
        let addr = match self.addr.as_ref() {
            None => return Ok(None),
            Some(addr) => addr,
        };
        let mut buf = Vec::new();
        self.storage.read(addr.clone(), &mut buf).await?;
        let node = Deser::default().deserialize(&buf)?;
        Ok(Some(LogContainer { addr, node }))
    }
    pub async fn first<T>(&self) -> Result<Option<LogNode<T>>, Error>
    where
        T: Deserialize,
    {
        let container = self.first_container().await?;
        Ok(container.map(|LogContainer { node, .. }| node))
    }
}
