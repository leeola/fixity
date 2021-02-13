pub use crate::map::PathSegment as MapSegment;
use {
    crate::{
        storage::{StorageRead, StorageWrite},
        Addr, Error,
    },
    std::fmt,
};
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path {
    segments: Vec<Segment>,
}
impl Path {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
    pub fn from_segments(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
    pub fn push<T>(&mut self, segment: T)
    where
        T: Into<Segment>,
    {
        self.segments.push(segment.into());
    }
    pub fn into_push<T>(mut self, segment: T) -> Self
    where
        T: Into<Segment>,
    {
        self.push(segment);
        self
    }
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    /// Pop the last [`Segment`] from the Path.
    pub fn pop(&mut self) -> Option<Segment> {
        self.segments.pop()
    }
    /// Returns `true` is this Path is empty.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    /// Reverses the order of Keys in the Path, in place.
    pub fn reverse(&mut self) {
        self.segments.reverse()
    }
    pub fn from_map<T>(map_segment: T) -> Self
    where
        T: Into<MapSegment>,
    {
        Self::new().into_map(map_segment)
    }
    pub fn push_map<T>(&mut self, map_segment: T)
    where
        T: Into<MapSegment>,
    {
        self.segments.push(Segment::Map(map_segment.into()));
    }
    pub fn into_map<T>(mut self, map_segment: T) -> Self
    where
        T: Into<MapSegment>,
    {
        self.push_map(map_segment);
        self
    }
    pub async fn resolve<S>(
        &self,
        storage: &S,
        root_addr: Option<Addr>,
    ) -> Result<Vec<Option<Addr>>, Error>
    where
        S: StorageRead,
    {
        let resolved_len = self.segments.len() + 1;
        let mut addr = match root_addr {
            Some(addr) => addr,
            None => return Ok(vec![None; resolved_len]),
        };
        let mut resolved_segs = Vec::with_capacity(resolved_len);
        resolved_segs.push(Some(addr.clone()));
        for seg in self.segments.iter() {
            match seg.resolve(storage, addr).await? {
                Some(resolved_addr) => {
                    resolved_segs.push(Some(resolved_addr.clone()));
                    addr = resolved_addr;
                }
                None => {
                    // resolve the remaining segments as None
                    // this will always be >= 1
                    resolved_segs.append(&mut vec![None; resolved_len - resolved_segs.len()]);
                    return Ok(resolved_segs);
                }
            }
        }
        Ok(resolved_segs)
    }
    pub async fn resolve_last<S>(
        &self,
        storage: &S,
        addr: Option<Addr>,
    ) -> Result<Option<Addr>, Error>
    where
        S: StorageRead,
    {
        let mut addr = match addr {
            Some(addr) => addr,
            None => return Ok(None),
        };
        for seg in self.segments.iter() {
            match seg.resolve(storage, dbg!(addr)).await? {
                Some(resolved_addr) => {
                    addr = resolved_addr;
                }
                None => {
                    return Ok(None);
                }
            }
        }
        Ok(Some(addr))
    }
    pub async fn update<S>(
        &self,
        storage: &S,
        resolved_addrs: Vec<Option<Addr>>,
        new_last_segment_addr: Addr,
    ) -> Result<Addr, Error>
    where
        S: StorageRead + StorageWrite,
    {
        let mut new_addr_cursor = new_last_segment_addr;
        for (seg_addr, seg) in resolved_addrs
            .into_iter()
            .rev()
            .skip(1)
            .zip(self.segments.iter().rev())
        {
            new_addr_cursor = seg.update(storage, seg_addr, new_addr_cursor).await?;
        }
        Ok(new_addr_cursor)
    }
}
impl<T> From<&[T]> for Path
where
    T: Clone + Into<Segment>,
{
    fn from(t: &[T]) -> Self {
        Self::from_segments(t.iter().map(|t| t.clone().into()).collect())
    }
}
impl IntoIterator for Path {
    type Item = Segment;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}
impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Path(\n")?;
        let iter = self.segments.iter();
        for seg in iter {
            f.write_str("    ")?;
            seg.fmt(f)?;
            f.write_str(",\n")?;
        }
        f.write_str(")")?;
        Ok(())
    }
}
impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Path(\n")?;
        let iter = self.segments.iter();
        for seg in iter {
            f.write_str("    ")?;
            seg.fmt(f)?;
            f.write_str(",\n")?;
        }
        f.write_str(")")?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
    Map(MapSegment),
}
impl Segment {
    pub fn map(self) -> Option<MapSegment> {
        let Segment::Map(s) = self;
        Some(s)
    }
}
impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Map(seg) => seg.fmt(f),
        }
    }
}
#[async_trait::async_trait]
impl<S> SegmentResolve<S> for Segment
where
    S: StorageRead,
{
    async fn resolve(&self, storage: &S, self_addr: Addr) -> Result<Option<Addr>, Error> {
        match self {
            Self::Map(seg) => seg.resolve(storage, self_addr).await,
        }
    }
}
#[async_trait::async_trait]
impl<S> SegmentUpdate<S> for Segment
where
    S: StorageRead + StorageWrite,
{
    async fn update(
        &self,
        storage: &S,
        self_addr: Option<Addr>,
        child_addr: Addr,
    ) -> Result<Addr, Error> {
        match self {
            Self::Map(seg) => seg.update(storage, self_addr, child_addr).await,
        }
    }
}
#[async_trait::async_trait]
pub trait SegmentResolve<S> {
    async fn resolve(&self, storage: &S, self_addr: Addr) -> Result<Option<Addr>, Error>;
}
#[async_trait::async_trait]
pub trait SegmentUpdate<S> {
    async fn update(
        &self,
        storage: &S,
        self_addr: Option<Addr>,
        child_addr: Addr,
    ) -> Result<Addr, Error>;
}
#[macro_export]
macro_rules! map_path {
    ( $( $x:expr ),* ) => {
        {
            let mut p = Path::new();
            $(
                p.push_map($x);
            )*
            p
        }
    };
}
