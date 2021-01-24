use {
    crate::{
        error::TypeError,
        path::{Path, Segment},
        primitive::{commitlog::CommitLog, prolly::refimpl},
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        workspace::{Guard, Status, Workspace},
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
    /// Drop the internal change cache that has not yet been staged or committed to storage.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map};
    /// let f = Fixity::memory();
    /// let mut m = f.map();
    /// m.insert("foo", "bar");
    /// m.clear();
    /// assert!(m.get("foo").await.unwrap().is_none());
    /// # }
    /// ```
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    /// Insert a value into the map to later be staged or committed.
    ///
    /// This value is not written to the store until [`Self::stage`] or [`Self::commit`]
    /// is called, but it can be retrived from the internal cache.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map};
    /// let f = Fixity::memory();
    /// let mut m_1 = f.map();
    /// let m_2 = f.map();
    /// m_1.insert("foo", "bar");
    /// // get pulls from in-memory cache, awaiting stage/commit.
    /// assert_eq!(m_1.get("foo").await.unwrap(), Some("bar".into()));
    /// // not yet written to storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), None);
    /// # }
    /// ```
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
}
impl<'f, S, W> Map<'f, S, W>
where
    S: StorageRead,
    W: Workspace,
{
    pub async fn get<K>(&self, key: K) -> Result<Option<Value>, Error>
    where
        K: Into<Key>,
    {
        let key = key.into();
        if let Some(refimpl::Change::Insert(value)) = self.cache.get(&key) {
            return Ok(Some(value.clone()));
        }
        let content_addr = self
            .workspace
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let reader = if let Some(content_addr) = content_addr {
            refimpl::Read::new(self.storage, content_addr)
        } else {
            return Ok(None);
        };
        reader.get(&key).await
    }
}
impl<'f, S, W> Map<'f, S, W>
where
    S: StorageRead + StorageWrite,
    W: Workspace,
{
    /// Write any changes to storage, staging them for a later commit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map};
    /// let f = Fixity::memory();
    /// let mut m_1 = f.map();
    /// let m_2 = f.map();
    /// m_1.insert("foo", "bar");
    /// // not yet written to storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), None);
    /// let _staged_addr = m_1.stage().await.unwrap();
    /// // now in storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), Some("bar".into()));
    /// # }
    pub async fn stage(&mut self) -> Result<Addr, Error> {
        if self.cache.is_empty() {
            return Err(Error::NoChangesToWrite);
        }
        // NIT: This drops the data on a failure - something we may want to tweak in the future.
        let kvs = mem::replace(&mut self.cache, HashMap::new()).into_iter();
        let workspace_guard = self.workspace.lock().await?;
        let staged_addr = workspace_guard
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let (resolved_path, old_self_addr) = match staged_addr {
            Some(staged_content) => {
                let resolved_path = self.path.resolve(&self.storage, staged_content).await?;
                let old_self_addr = resolved_path.last().cloned().unwrap_or(None);
                (resolved_path, old_self_addr)
            }
            None => (vec![None; self.path.len()], None),
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
        let new_staged_content = self
            .path
            .update(&self.storage, resolved_path, new_self_addr)
            .await?;
        workspace_guard.stage(new_staged_content.clone()).await?;
        Ok(new_staged_content)
    }
    /// Write any [staged](Self::stage) changes at the current [`Path`] into the workspace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map};
    /// let f = Fixity::memory();
    /// let mut m_1 = f.map();
    /// let m_2 = f.map();
    /// m_1.insert("foo", "bar");
    /// // not yet written to storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), None);
    /// m_1.stage().await.unwrap();
    /// m_1.commit().await.unwrap();
    /// // now in storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), Some("bar".into()));
    /// # }
    pub async fn commit(&mut self) -> Result<Addr, Error> {
        let workspace_guard = self.workspace.lock().await?;
        let (commit_addr, staged_addr) = match workspace_guard.status().await? {
            Status::InitStaged { staged_content, .. } => (None, staged_content),
            Status::Staged {
                commit,
                staged_content,
                ..
            } => (Some(commit), staged_content),
            Status::Detached(_) => return Err(Error::DetachedHead),
            Status::Init { .. } | Status::Clean { .. } => {
                return Err(Error::NoStageToCommit);
            }
        };
        let mut commit_log = CommitLog::new(self.storage, commit_addr);
        let commit_addr = commit_log.append(staged_addr).await?;
        workspace_guard.commit(commit_addr.clone()).await?;
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
    use crate::Fixity;
    #[tokio::test]
    async fn write_to_storage() {
        let f = Fixity::memory();
        let mut m_1 = f.map();
        let m_2 = f.map();
        m_1.insert("foo", "bar");
        // not yet written to storage.
        assert_eq!(m_2.get("foo").await.unwrap(), None);
        m_1.stage().await.unwrap();
        m_1.commit().await.unwrap();
        // now in storage.
        assert_eq!(m_2.get("foo").await.unwrap(), Some("bar".into()));
    }
}
