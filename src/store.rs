use {
    crate::{Commit, Id, Result},
    std::io::{Read, Write},
};
pub trait Store {
    fn put(&self, bytes: &mut dyn Read, hashes: &mut dyn Write) -> Result<usize>;
    // fn new() -> Id;
    // fn push(content: T, id: Option<Id>) -> Result<Commit>;
    // fn clone() -> ();
}
