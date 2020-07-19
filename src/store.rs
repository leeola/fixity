use {
    crate::{Addr, Commit, Id, Result},
    std::io::{Read, Write},
};
pub trait Store {
    fn put_chunk(&self, chunk: &dyn AsRef<[u8]>) -> Result<Addr>;
    fn put(&self, bytes: &mut dyn Read) -> Result<Addr>;
    // fn new() -> Id;
    // fn push(content: T, id: Option<Id>) -> Result<Commit>;
    // fn clone() -> ();
}
