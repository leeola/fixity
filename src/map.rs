use {
    crate::{
        error::TypeError,
        path::{Path, Segment},
        primitive::prolly::refimpl,
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
    cache: HashMap<Key, refimpl::Change>,
}
impl<'f, S, W> Map<'f, S, W> {
    pub fn new(storage: &'f S, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
            cache: HashMap::new(),
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
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        self.cache
            .insert(key.into(), refimpl::Change::Insert(value.into()));
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
    W: Workspace,
{
    pub async fn stage(&mut self) -> Result<Addr, Error> {
        todo!("map stage")
    }
    pub async fn commit(&mut self) -> Result<Addr, Error> {
        //     let head = Head::open(self.fixity_dir.as_path(), self.workspace.as_str()).await?;
        let kvs = mem::replace(&mut self.cache, HashMap::new()).into_iter();
        let head_addr = self.workspace.head().await?;
        dbg!(&head_addr);
        let addr = if let Some(addr) = head_addr {
            let kvs = kvs.collect::<Vec<_>>();
            refimpl::Update::new(self.storage, addr)
                .with_vec(kvs)
                .await?
        } else {
            let kvs = kvs
                .filter_map(|(k, change)| match change {
                    refimpl::Change::Insert(v) => Some((k, v)),
                    refimpl::Change::Remove => None,
                })
                .collect::<Vec<_>>();
            refimpl::Create::new(self.storage).with_vec(kvs).await?
        };
        dbg!(&addr);
        //     let commit_log = CommitLog::new(&self.storage, head.addr());
        todo!("map commit")
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
        let mut m = f.map();
        let expected = Value::from("bar");
        dbg!(m.insert("foo", expected));
        dbg!(m.commit().await.unwrap());
        // dbg!(m.get("foo").await.unwrap());
    }
}
