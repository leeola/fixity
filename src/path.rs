use {
    crate::{
        map::MapSegment,
        storage::{StorageRead, StorageWrite},
        Addr, Error,
    },
    dyn_clone::DynClone,
    std::fmt::Debug,
};
#[derive(Debug, Default)]
pub struct Path<S> {
    segments: Vec<Box<dyn Segment<S>>>,
}
impl<S> Path<S> {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
    pub fn push<T>(&mut self, segment: T)
    where
        T: Segment<S> + 'static,
    {
        self.segments.push(Box::new(segment));
    }
    pub fn len(&self) -> usize {
        self.segments.len()
    }
}
impl<S> Path<S>
where
    S: StorageRead + StorageWrite,
{
    pub fn push_map<T>(&mut self, map_segment: T)
    where
        T: Into<MapSegment>,
    {
        self.segments.push(Box::new(map_segment.into()));
    }
    pub fn into_map<T>(mut self, map_segment: T) -> Self
    where
        T: Into<MapSegment>,
    {
        self.push(map_segment.into());
        self
    }
    pub async fn resolve(&self, storage: &S, mut addr: Addr) -> Result<Vec<Option<Addr>>, Error> {
        let mut resolved_segs = Vec::new();
        for seg in self.segments.iter() {
            match seg.resolve(storage, addr).await? {
                Some(resolved_addr) => {
                    resolved_segs.push(Some(resolved_addr.clone()));
                    addr = resolved_addr;
                }
                None => {
                    resolved_segs.push(None);
                    return Ok(resolved_segs);
                }
            }
        }
        Ok(resolved_segs)
    }
    pub async fn update(
        &self,
        storage: &S,
        resolved_addrs: Vec<Option<Addr>>,
        mut new_addr: Addr,
    ) -> Result<Addr, Error> {
        for (seg_addr, seg) in resolved_addrs.into_iter().zip(self.segments.iter()) {
            new_addr = seg.update(storage, seg_addr, new_addr).await?;
        }
        Ok(new_addr)
    }
}
// Implementing clone manually because the Path<S> constraint assumes `S: Clone`, but that's
// not needed.
impl<S> Clone for Path<S> {
    fn clone(&self) -> Self {
        Self {
            segments: self.segments.clone(),
        }
    }
}
#[async_trait::async_trait]
pub trait Segment<S>: Debug + DynClone {
    async fn resolve(&self, storage: &S, self_addr: Addr) -> Result<Option<Addr>, Error>;
    async fn update(
        &self,
        storage: &S,
        self_addr: Option<Addr>,
        value_addr: Addr,
    ) -> Result<Addr, Error>;
}
dyn_clone::clone_trait_object!(<S> Segment<S>);
