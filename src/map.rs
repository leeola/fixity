use {
    crate::{
        error::TypeError,
        path::{Path, Segment},
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        workspace::Workspace,
        Addr, Error,
    },
    std::{collections::HashMap, mem},
};
pub struct Map<'f, S, W> {
    storage: &'f S,
    workspace: &'f W,
    path: Path,
}
impl<'f, S, W> Map<'f, S, W> {
    pub fn new<K>(storage: &'f S, workspace: &'f W, path: K) -> Self
    // TODO: Make Key some form of Vec<Key> or KeyPath
    where
        K: Into<Key>,
    {
        Self {
            storage,
            workspace,
            path: todo!("map path"),
        }
    }
    pub fn map<K>(&self, key_path: K) -> Self
    // TODO: Make Key some form of Vec<Key> or KeyPath
    where
        K: Into<Key>,
    {
        // Self::new(&self.storage, &self.workspace, self.path.with(key_path))
        // TODO: merge key_paths.
        Self::new(&self.storage, &self.workspace, key_path)
    }
}
impl<'f, S, W> Map<'f, S, W>
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
impl<'f, S, W> Map<'f, S, W>
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
#[derive(Debug, Clone)]
pub struct MapSegment {
    key: Key,
}
impl Segment for MapSegment {
    fn resolve(&self, addr: Addr) -> Result<Option<Addr>, Error> {
        todo!("map resolve")
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::Fixity};
    #[tokio::test]
    async fn poc() {
        let f = Fixity::test();
        let m = f.map(None);
        let expected = "bar".into();
        dbg!(m.put("foo", expected).await.unwrap());
        dbg!(m.get("foo").await.unwrap());
    }
}
