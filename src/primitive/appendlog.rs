use crate::{
    cache::{CacheRead, CacheWrite},
    deser::{Deser, Deserialize, Serialize},
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
pub struct AppendLog<'s, C> {
    storage: &'s C,
    addr: Option<Addr>,
}
impl<'s, C> AppendLog<'s, C> {
    pub fn new(storage: &'s C, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
}
impl<'s, C> AppendLog<'s, C>
where
    C: CacheRead + CacheWrite,
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
            Deser::default().to_vec(&node)?
        };
        let addr = Addr::hash(&buf);
        self.storage.write(addr.clone(), &*buf).await?;
        let _ = self.addr.replace(addr.clone());
        Ok(addr)
    }
}
impl<'s, C> AppendLog<'s, C>
where
    C: CacheRead,
{
    pub async fn first_container<T>(&self) -> Result<Option<LogContainer<'_, LogNode<T>>>, Error>
    where
        T: Deserialize,
    {
        let addr = match self.addr.as_ref() {
            None => return Ok(None),
            Some(addr) => addr,
        };
        let buf = self.storage.read(addr.clone()).await?;
        let node = Deser::default().from_slice(buf.as_ref())?;
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
