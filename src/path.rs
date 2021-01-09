use crate::{Addr, Error};

pub struct Path {
    segments: Vec<Box<dyn Segment>>,
}
pub trait Segment {
    fn resolve(&self, addr: Addr) -> Result<Option<Addr>, Error>;
}
use crate::value::Key;
pub struct MapSegment {
    key: Key,
}
