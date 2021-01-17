use {
    crate::{
        error::TypeError,
        path::{Path, Segment},
        primitive::{commitlog::CommitLog, prolly::refimpl},
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
    path: Path<S>,
    cache: HashMap<Key, refimpl::Change>,
}
impl<'f, S, W> Map<'f, S, W> {
    pub fn new(storage: &'f S, workspace: &'f W, path: Path<S>) -> Self {
        Self {
            storage,
            workspace,
            path,
            cache: HashMap::new(),
        }
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
    S: StorageRead + StorageWrite,
{
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
    pub async fn get<K>(&self, key: K) -> Result<Option<Value>, Error>
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
        let kvs = mem::replace(&mut self.cache, HashMap::new()).into_iter();
        let head_addr = self.workspace.head().await?;
        dbg!(&head_addr);
        let mut commit_log = CommitLog::new(self.storage, head_addr);
        let (resolved_path, old_self_addr) = if let Some(commit) = commit_log.first().await? {
            let root_addr = commit.content;
            let resolved_path = self.path.resolve(&self.storage, root_addr).await?;
            let old_self_addr = resolved_path.last().cloned().unwrap_or(None);
            (resolved_path, old_self_addr)
        } else {
            (vec![None; self.path.len()], None)
        };
        let new_self_addr = if let Some(self_addr) = old_self_addr {
            let kvs = kvs.collect::<Vec<_>>();
            refimpl::Update::new(self.storage, self_addr)
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
        let root_addr = self
            .path
            .update(&self.storage, resolved_path, new_self_addr)
            .await?;
        let commit_addr = commit_log.append(root_addr).await?;
        self.workspace.append(commit_addr.clone()).await?;
        Ok(commit_addr)
    }
}
#[derive(Debug, Clone)]
pub struct MapSegment {
    key: Key,
}
#[async_trait::async_trait]
impl<'f, S> Segment<S> for MapSegment
where
    S: StorageRead + StorageWrite,
{
    async fn resolve(&self, storage: &S, self_addr: Addr) -> Result<Option<Addr>, Error> {
        let reader = refimpl::Read::new(storage, self_addr);
        let value = match reader.get(&self.key).await? {
            Some(v) => v,
            None => return Ok(None),
        };
        let addr = match value {
            Value::Addr(addr) => addr,
            _ => {
                return Err(Error::Type(TypeError::UnexpectedValueVariant {
                    at_segment: Some(self.key.to_string()),
                    // addr moved, not sure it's worth prematurely cloning for the failure state.
                    at_addr: None,
                }));
            }
        };
        Ok(Some(addr))
    }
    async fn update(
        &self,
        storage: &S,
        self_addr: Option<Addr>,
        child_addr: Addr,
    ) -> Result<Addr, Error> {
        if let Some(self_addr) = self_addr {
            let kvs = vec![(
                self.key.clone(),
                refimpl::Change::Insert(Value::Addr(child_addr)),
            )];
            refimpl::Update::new(storage, self_addr).with_vec(kvs).await
        } else {
            let kvs = vec![(self.key.clone(), Value::Addr(child_addr))];
            refimpl::Create::new(storage).with_vec(kvs).await
        }
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
        m.insert("foo", expected.clone());
        dbg!(m.commit().await.unwrap());
        dbg!(m.get("foo").await.unwrap());
        assert_eq!(m.get("foo").await.unwrap(), Some(expected));
    }
}
