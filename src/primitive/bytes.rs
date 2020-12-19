use {
    crate::{
        error::TypeError,
        primitive::{Build, Flush, GetAddr, InsertAddr},
        prolly::refimpl,
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        Addr, Error,
    },
    std::{collections::HashMap, mem},
};
pub struct Bytes<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
}
impl<'s, S> Bytes<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
    pub fn read(&self, path: Path) -> () {
        todo!()
    }
}
pub struct ByteReader<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
}
impl<'s, S> ByteReader<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
}
