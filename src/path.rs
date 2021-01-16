use {
    crate::{map::MapSegment, storage::StorageRead, Addr, Error},
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
}
impl<S> Path<S>
where
    S: StorageRead,
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
    pub async fn resolve(&self, storage: &S, mut addr: Addr) -> Result<Option<Addr>, Error> {
        for seg in self.segments.iter() {
            addr = match seg.resolve(storage, addr).await? {
                Some(addr) => addr,
                None => return Ok(None),
            };
        }
        Ok(Some(addr))
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
    async fn resolve(&self, storage: &S, addr: Addr) -> Result<Option<Addr>, Error>;
    // fn update(&self, addr: Addr) -> Result<Addr, Error>;
}
dyn_clone::clone_trait_object!(<S> Segment<S>);
