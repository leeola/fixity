use {
    crate::{
        error::TypeError,
        path::Path,
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        workspace::Workspace,
        Addr, Error,
    },
    std::{collections::HashMap, mem},
};
pub struct Map<'f, S> {
    storage: &'f S,
    workspace: &'f Workspace,
    path: Path,
}
impl<'f, S> Map<'f, S> {
    pub fn new(storage: &'f S, workspace: &'f Workspace, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
    pub fn map<K>(&self, key_path: K) -> Self
    // TODO: Make Key some form of Vec<Key> or KeyPath
    where
        K: Into<Key>,
    {
        todo!("map map")
    }
}
impl<'f, S> Map<'f, S>
where
    S: StorageRead,
{
    pub async fn get<K>(&self, key: K) -> Result<Value, Error>
    where
        K: Into<Key>,
    {
        todo!("get")
    }
}
impl<'f, S> Map<'f, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn put<K, V>(&self, key: K, value: V) -> Result<Addr, Error>
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        todo!("put")
    }
}
