use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            misc::range_ext::{OwnedRangeBounds, RangeBoundsExt},
            primitive::{commitlog::CommitLog, hash_set::refimpl},
            workspace::{AsWorkspaceRef, Guard, Status, Workspace},
        },
        error::Type as TypeError,
        // TODO: move SegmentUpdate / SegmentUpdate to core
        path::{SegmentResolve, SegmentUpdate},
        Addr,
        Error,
        Key,
        Path,
        Value,
    },
    std::{fmt, mem, ops::Bound},
};
pub struct HashSet<'f, C, W> {
    storage: &'f C,
    workspace: &'f W,
    path: Path,
}
impl<'f, C, W> HashSet<'f, C, W> {
    pub fn new(storage: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathSegment {
    pub addr: Addr,
}
impl PathSegment {
    pub fn new<T: Into<Addr>>(t: T) -> Self {
        Self { addr: t.into() }
    }
}
impl fmt::Debug for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashSet(")?;
        self.addr.fmt(f)?;
        f.write_str(")")
    }
}
impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashSet(")?;
        self.addr.fmt(f)?;
        f.write_str(")")
    }
}
#[async_trait::async_trait]
impl<C> SegmentResolve<C> for PathSegment
where
    C: CacheRead,
{
    async fn resolve(&self, storage: &C, self_addr: Addr) -> Result<Option<Addr>, Error> {
        // Ensure that the Addr is already in the Set. While HashSet is awkward in this
        // interface, we want to resolve a valid, existing path - if it's not in the Set
        // there's nothing to resolve - it's an invalid/non-existent Path.
        let reader = refimpl::Read::new(storage, self_addr);
        let value = Value::from(self.addr.clone());
        if !reader.contains(&value).await? {
            return Ok(None);
        }
        // As a cheeky optimization, we clone Addr and store it in a value for the contains() check,
        // and then pull it back out - avoiding a second allocation. ... it's just a small
        // array in a non-hot path, probably not worth this.
        match value {
            Value::Addr(addr) => Ok(Some(addr)),
            _ => unreachable!("Value was assigned as Addr above"),
        }
    }
}
impl<T> From<T> for PathSegment
where
    T: Into<Addr>,
{
    fn from(t: T) -> Self {
        Self { addr: t.into() }
    }
}
