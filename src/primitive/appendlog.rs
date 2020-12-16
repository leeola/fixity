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
pub struct Log<T> {
    inner: T,
    next: Option<Addr>,
}
pub struct AppendLog<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
}
impl<'s, S> AppendLog<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
}
impl<'s, S> AppendLog<'s, S>
where
    S: StorageWrite,
{
    pub async fn append<T>(&mut self, t: T) -> Result<Addr, Error> {
        todo!("appendlog .. append")
    }
}
impl<'s, S> AppendLog<'s, S>
where
    S: StorageRead,
{
    pub async fn first<T>(&self) -> Result<Option<Log<T>>, Error> {
        todo!("get rel")
    }
    pub async fn get_to_rel(&self, _i: usize) -> Result<Vec<T>, Error> {
        todo!("get to rel")
    }
}
