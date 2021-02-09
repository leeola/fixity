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
    tokio::io::{self, AsyncRead, AsyncWrite},
};
pub struct Bytes<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
}
impl<'s, S> Bytes<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
    pub fn read<W>(&self, w: W) -> Result<(), Error>
    where
        S: StorageRead,
        W: AsyncWrite + Unpin + Send,
    {
        todo!("bytes read")
    }
    pub async fn write<R>(&self, r: R) -> Result<Addr, Error>
    where
        S: StorageWrite,
        R: AsyncRead + Unpin + Send;
    {
        todo!("bytes write")
    }
}
