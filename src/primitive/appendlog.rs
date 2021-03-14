use crate::{
    cache::{CacheRead, CacheWrite, OwnedRef, Structured},
    primitive::commitlog,
    storage::Error as StorageError,
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
#[derive(Debug, Clone)]
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
impl From<commitlog::CommitNode> for LogInnerType {
    fn from(n: commitlog::CommitNode) -> LogInnerType {
        LogInnerType::Commit(n)
    }
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
pub trait LogNodeFrom<T>: Sized {
    fn log_node_from(t: T) -> Option<LogNode<Self>>;
}
impl LogNodeFrom<Structured> for commitlog::CommitNode {
    fn log_node_from(t: Structured) -> Option<LogNode<Self>> {
        if let Structured::CommitLogNode(node) = t {
            Some(node)
        } else {
            None
        }
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
        T: Into<LogInnerType> + Send,
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
        T: LogNodeFrom<Structured>,
    {
        let addr = match self.addr.as_ref() {
            Some(addr) => addr,
            None => return Ok(None),
        };
        let owned_ref = self.cache.read_structured(addr).await?;
        // TODO: the design of AppendLog makes using OwnedRef::Ref really awkward,
        // and needs to be redesigned. However appendlog is not heavily used currently,
        // only commits, so it's a low perf impact to just own the value immediately.
        let node = T::log_node_from(owned_ref.into_owned_structured())
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            // Also, this is going to likely be a CacheError in the future?
            .ok_or_else(|| StorageError::Unhandled {
                message: "misaligned cache types".to_owned(),
            })?;
        Ok(Some(LogContainer { addr, node }))
    }
    pub async fn first<T>(&self) -> Result<Option<LogNode<T>>, Error>
    where
        T: LogNodeFrom<Structured>,
    {
        let container = self.first_container().await?;
        Ok(container.map(|LogContainer { node, .. }| node))
    }
}
