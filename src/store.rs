use {
    crate::{Addr, Commit, Id, Result},
    std::io::{Read, Write},
    async_trait::async_trait,
};
#[async_trait]
pub trait Store {
    // fn put_read(&self, bytes: &mut dyn Read) -> Result<Addr>;
    // fn new() -> Id;
    // fn push(content: T, id: Option<Id>) -> Result<Commit>;
    // fn clone() -> ();
}
