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
    pub fn new(storage: &'f S, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
    pub fn map<K>(&self, key: K) -> Self
    where
        K: Into<Key>,
    {
        Self::new(
            &self.storage,
            &self.workspace,
            self.path.clone().into_map(MapSegment { key: key.into() }),
        )
    }
    pub fn into_map<K>(mut self, key: K) -> Self
    where
        K: Into<Key>,
    {
        self.path.push_map(MapSegment { key: key.into() });
        self
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
        let m = f.map();
        let expected = Value::from("bar");
        dbg!(m.put("foo", expected).await.unwrap());
        dbg!(m.get("foo").await.unwrap());
    }
}
