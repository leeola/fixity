use {
    crate::{Addr, Error},
    dyn_clone::DynClone,
    std::fmt::Debug,
};
#[derive(Debug, Clone)]
pub struct Path {
    segments: Vec<Box<dyn Segment>>,
}
pub trait Segment: Debug + DynClone {
    fn resolve(&self, addr: Addr) -> Result<Option<Addr>, Error>;
    // fn update(&self, addr: Addr) -> Result<Addr, Error>;
}
dyn_clone::clone_trait_object!(Segment);
