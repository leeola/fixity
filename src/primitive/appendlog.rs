use crate::{
    cache::{CacheRead, CacheWrite, Structured},
    deser::{Deser, Deserialize, Serialize},
    primitive::commitlog,
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
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct LogNode<T> {
    pub inner: T,
    pub prev: Option<Addr>,
}
/// An awkward interface, needed to allow the `T` in `LogNode<T>` to
/// resolve to a concrete [`Structured`] variant.
///
/// With this type, `AppendLog` extensions like `CommitLog` can implement `Into<LogInnerType>`
/// and allow AppendLog to do the conversion to a proper `Structured` variant.
#[derive(Debug)]
pub enum LogInnerType {
    Commit(commitlog::CommitNode),
}
impl<T> From<LogNode<T>> for Structured
where
    T: Into<LogInnerType>,
{
    fn from(log_node: LogNode<T>) -> Self {
        let LogNode { inner, prev } = log_node;
        // single variant for now, idiomatic..
        let LogInnerType::Commit(inner) = inner.into();
        Structured::CommitLogNode(LogNode { inner, prev })
    }
}
pub struct AppendLog<'s, C> {
    cache: &'s C,
    addr: Option<Addr>,
}
impl<'s, C> AppendLog<'s, C> {
    pub fn new(cache: &'s C, addr: Option<Addr>) -> Self {
        Self { cache, addr }
    }
}
impl<'s, C> AppendLog<'s, C>
where
    C: CacheRead + CacheWrite,
{
    pub async fn append<T>(&mut self, inner: T) -> Result<Addr, Error>
    where
        T: Into<LogInnerType>,
    {
        let node = LogNode {
            inner,
            prev: self.addr.clone(),
        };
        // let addr = Addr::hash(&buf);
        let addr = self.cache.write_structured(node).await?;
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
            Some(addr) => addr,
            None => return Ok(None),
        };
        let buf = self.cache.read(addr).await?;
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
