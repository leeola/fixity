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
pub struct AppendLog<'s, S, T> {
    storage: &'s S,
    addr: Option<Addr>,
    // reader: Option<refimpl::Read<'s, S>>,
    /// A single item "queue" for `T` waiting on being flushed to storage.
    flush_queue: Option<T>,
}
impl<'s, S, T> AppendLog<'s, S, T> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self {
            storage,
            addr,
            // reader,
            flush_queue: None,
        }
    }
    pub fn insert<V>(&mut self, v: V) -> Option<T>
    where
        V: Into<T>,
    {
        self.flush_queue.replace(v.into())
    }
}
impl<'s, S, T> AppendLog<'s, S, T>
where
    S: StorageRead,
{
    pub async fn get_rel(&self, i: usize) -> Result<Option<T>, Error> {
        todo!()
    }
    pub async fn get_to_rel(&self, i: usize) -> Result<Vec<T>, Error> {
        todo!()
    }
}
