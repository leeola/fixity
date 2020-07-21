use {
    crate::{Fixity, Result},
    fastcdc::Chunk,
};
#[derive(Debug)]
pub struct Node {
    depth: usize,
    max_children: usize,
    size: usize,
}
impl Node {
    pub fn new<S>(
        fixity: Fixity<S>,
        data: &[u8], impl Iterator<Item
iter: impl Iterator<Item = Chunk>,
    ) -> Result<Self> {
        todo!()
    }
}
