use {
    crate::{map::MapSegment, Addr, Error},
    dyn_clone::DynClone,
    std::fmt::Debug,
};
#[derive(Debug, Clone, Default)]
pub struct Path {
    segments: Vec<Box<dyn Segment>>,
}
impl Path {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push<T>(&mut self, segment: T)
    where
        T: Segment + 'static,
    {
        self.segments.push(Box::new(segment));
    }
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
}
pub trait Segment: Debug + DynClone {
    fn resolve(&self, addr: Addr) -> Result<Option<Addr>, Error>;
    // fn update(&self, addr: Addr) -> Result<Addr, Error>;
}
dyn_clone::clone_trait_object!(Segment);
