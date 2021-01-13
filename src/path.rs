use {
    crate::{Addr, Error},
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
        T: Segment,
    {
        self.segments.push(Box::new(segment));
    }
}
pub trait Segment: Debug + DynClone {
    fn resolve(&self, addr: Addr) -> Result<Option<Addr>, Error>;
    // fn update(&self, addr: Addr) -> Result<Addr, Error>;
}
dyn_clone::clone_trait_object!(Segment);
