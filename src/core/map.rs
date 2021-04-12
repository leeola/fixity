use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            misc::range_ext::{OwnedRangeBounds, RangeBoundsExt},
            primitive::{commitlog::CommitLog, prollytree::refimpl},
            workspace::{AsWorkspaceRef, Guard, Status, Workspace},
        },
        path::{MapSegment, Path},
        Addr, Error, Key, Value,
    },
    std::{collections::HashMap, mem, ops::Bound},
};
pub struct Map<'f, C, W> {
    storage: &'f C,
    workspace: &'f W,
    path: Path,
}
impl<'f, C, W> Map<'f, C, W> {
    pub fn new(storage: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
    pub fn batch(&self) -> BatchMap<'f, C, W> {
        BatchMap::new(self.storage, self.workspace, self.path.clone())
    }
    /// The core implementation of [`insert`](crate::Map::insert).
    pub async fn insert<K, V>(&self, key: K, value: V) -> Result<Addr, Error>
    where
        K: Into<Key>,
        V: Into<Value>,
        C: CacheRead + CacheWrite,
        W: Workspace,
    {
        let workspace_guard = self.workspace.lock().await?;
        let root_content_addr = workspace_guard
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let resolved_path = self
            .path
            .resolve(self.storage, root_content_addr.clone())
            .await?;
        let old_self_addr = resolved_path
            .last()
            .cloned()
            .expect("resolved Path has zero len");
        let new_self_addr = if let Some(self_addr) = old_self_addr {
            refimpl::Update::new(self.storage, self_addr)
                .with_vec(vec![(key.into(), refimpl::Change::Insert(value.into()))])
                .await?
        } else {
            refimpl::Create::new(self.storage)
                .with_vec(vec![(key.into(), value.into())])
                .await?
        };
        let new_staged_content = self
            .path
            .update(self.storage, resolved_path, new_self_addr)
            .await?;
        workspace_guard.stage(new_staged_content.clone()).await?;
        Ok(new_staged_content)
    }
    /// The core implementation of [`get`](crate::Map::get).
    pub async fn get<K>(&self, key: K) -> Result<Option<Value>, Error>
    where
        K: Into<Key>,
        C: CacheRead + CacheWrite,
        W: Workspace,
    {
        let key = key.into();
        let content_addr = self
            .workspace
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let content_addr = self.path.resolve_last(self.storage, content_addr).await?;
        let reader = if let Some(content_addr) = content_addr {
            refimpl::Read::new(self.storage, content_addr)
        } else {
            return Ok(None);
        };
        reader.get(&key).await
    }
    pub async fn iter<R>(
        &self,
        range: R,
    ) -> Result<Box<dyn Iterator<Item = Result<(Key, Value), Error>>>, Error>
    where
        W: Workspace,
        C: CacheRead + CacheWrite,
        R: RangeBoundsExt<Key>,
    {
        let content_addr = self
            .workspace
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let content_addr = self.path.resolve_last(self.storage, content_addr).await?;
        let reader = if let Some(content_addr) = content_addr {
            refimpl::Read::new(self.storage, content_addr)
        } else {
            return Ok(Box::new(Vec::new().into_iter()));
        };
        let OwnedRangeBounds { start, end } = range.into_bounds();
        let iter = reader
            .to_vec()
            .await?
            .into_iter()
            .filter(move |(k, _)| match &start {
                Bound::Included(start) => k >= start,
                Bound::Excluded(start) => k > start,
                Bound::Unbounded => true,
            })
            .take_while(move |(k, _)| match &end {
                Bound::Included(end) => k >= end,
                Bound::Excluded(end) => k > end,
                Bound::Unbounded => true,
            })
            .map(Ok);
        Ok(Box::new(iter))
    }
}
pub struct BatchMap<'f, C, W> {
    storage: &'f C,
    workspace: &'f W,
    path: Path,
    cache: HashMap<Key, refimpl::Change>,
}
impl<'f, C, W> BatchMap<'f, C, W> {
    pub fn new(storage: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
            cache: HashMap::new(),
        }
    }
    /// The core implementation of [`clear`](crate::BatchMap::clear).
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    /// The core implementation of [`insert`](crate::BatchMap::insert).
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        // TODO: move multi-stepped-insertion behavior to its own struct, such that
        // `BatchMap` becomes an always-clean interface.
        self.cache
            .insert(key.into(), refimpl::Change::Insert(value.into()));
    }
    pub async fn iter<R>(
        &self,
        range: R,
    ) -> Result<Box<dyn Iterator<Item = Result<(Key, Value), Error>>>, Error>
    where
        W: Workspace,
        C: CacheRead + CacheWrite,
        R: RangeBoundsExt<Key>,
    {
        let content_addr = self
            .workspace
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let content_addr = self.path.resolve_last(self.storage, content_addr).await?;
        let reader = if let Some(content_addr) = content_addr {
            refimpl::Read::new(self.storage, content_addr)
        } else {
            return Ok(Box::new(Vec::new().into_iter()));
        };
        let OwnedRangeBounds { start, end } = range.into_bounds();
        let iter = reader
            .to_vec()
            .await?
            .into_iter()
            .filter(move |(k, _)| match &start {
                Bound::Included(start) => k >= start,
                Bound::Excluded(start) => k > start,
                Bound::Unbounded => true,
            })
            .take_while(move |(k, _)| match &end {
                Bound::Included(end) => k >= end,
                Bound::Excluded(end) => k > end,
                Bound::Unbounded => true,
            })
            .map(Ok);
        Ok(Box::new(iter))
    }
}
impl<'f, C, W> BatchMap<'f, C, W>
where
    C: CacheRead + CacheWrite,
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
impl<'f, C, W> BatchMap<'f, C, W>
where
    C: CacheRead + CacheWrite,
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
        let content_addr = self.path.resolve_last(self.storage, content_addr).await?;
        let reader = if let Some(content_addr) = content_addr {
            refimpl::Read::new(self.storage, content_addr)
        } else {
            return Ok(None);
        };
        reader.get(&key).await
    }
    /// The core implementation of [`stage`](crate::BatchMap::stage).
    pub async fn stage(&mut self) -> Result<Addr, Error> {
        if self.cache.is_empty() {
            return Err(Error::NoChangesToWrite);
        }
        // NIT: This drops the data on a failure - something we may want to tweak in the future.
        let kvs = mem::replace(&mut self.cache, HashMap::new()).into_iter();
        let workspace_guard = self.workspace.lock().await?;
        let root_content_addr = workspace_guard
            .status()
            .await?
            .content_addr(self.storage)
            .await?;
        let resolved_path = self
            .path
            .resolve(self.storage, root_content_addr.clone())
            .await?;
        let old_self_addr = resolved_path
            .last()
            .cloned()
            .expect("resolved Path has zero len");
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
            .update(self.storage, resolved_path, new_self_addr)
            .await?;
        workspace_guard.stage(new_staged_content.clone()).await?;
        Ok(new_staged_content)
    }
    /// The core implementation of [`stage`](crate::BatchMap::stage).
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
            },
        };
        let mut commit_log = CommitLog::new(self.storage, commit_addr);
        let commit_addr = commit_log.append(staged_addr).await?;
        workspace_guard.commit(commit_addr.clone()).await?;
        Ok(commit_addr)
    }
}
impl<C, W> AsWorkspaceRef for Map<'_, C, W>
where
    W: Workspace,
{
    type Workspace = W;
    fn as_workspace_ref(&self) -> &Self::Workspace {
        &self.workspace
    }
}
impl<C, W> AsCacheRef for Map<'_, C, W>
where
    C: CacheRead + CacheWrite,
{
    type Cache = C;
    fn as_cache_ref(&self) -> &Self::Cache {
        &self.storage
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::core::Fixity};
    #[tokio::test]
    async fn write_to_root() {
        let f = Fixity::memory();
        let mut m_1 = f.map(Path::new()).batch();
        let m_2 = f.map(Path::new()).batch();
        m_1.insert("foo", "bar");
        assert_eq!(m_2.get("foo").await.unwrap(), None);
        m_1.stage().await.unwrap();
        m_1.commit().await.unwrap();
        assert_eq!(m_2.get("foo").await.unwrap(), Some("bar".into()));
    }
    #[tokio::test]
    async fn write_to_path_single() {
        let f = Fixity::memory();
        let mut m_1 = f.map(Path::from_map("foo")).batch();
        m_1.insert("bang", "boom");
        m_1.stage().await.unwrap();
        m_1.commit().await.unwrap();
        let m_2 = f.map(Path::new()).batch();
        let foo_value = m_2.get("foo").await.unwrap().unwrap();
        assert!(matches!(foo_value, Value::Addr(_)));
        assert_eq!(
            m_2.map("foo").get("bang").await.unwrap(),
            Some("boom".into())
        );
    }
    #[tokio::test]
    async fn write_to_path_double() {
        let f = Fixity::memory();
        let mut m_1 = f.map(Path::new()).batch().into_map("foo").into_map("bar");
        m_1.insert("bang", "boom");
        m_1.stage().await.unwrap();
        m_1.commit().await.unwrap();
        let m_2 = f.map(Path::new()).batch();
        println!("{:?}", m_2.get("foo").await.unwrap());
        println!("{:?}", m_2.get("bar").await.unwrap());
        let foo_value = m_2.get("foo").await.unwrap().unwrap();
        println!("{:?}", &foo_value);
        assert!(matches!(foo_value, Value::Addr(_)));
        let m_2 = m_2.into_map("foo");
        let bar_value = m_2.get("bar").await.unwrap().unwrap();
        assert!(matches!(bar_value, Value::Addr(_)));
        let m_2 = m_2.map("bar");
        assert_eq!(m_2.get("bang").await.unwrap(), Some("boom".into()));
    }
    #[tokio::test]
    async fn multi_value() {
        let f = Fixity::memory();
        let mut m = f.map(Path::new()).batch();
        m.insert("foo", "fooval");
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        m.stage().await.unwrap();
        m.insert("bar", "barval");
        m.stage().await.unwrap();
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        assert_eq!(m.get("bar").await.unwrap(), Some("barval".into()));
        m.commit().await.unwrap();
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        assert_eq!(m.get("bar").await.unwrap(), Some("barval".into()));
    }
    #[tokio::test]
    async fn nested_multi_value() {
        let f = Fixity::memory();
        let mut m = f.map(Path::new()).batch();
        m.insert("foo", "fooval");
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        m.stage().await.unwrap();
        m.insert("bar", "barval");
        m.stage().await.unwrap();
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        assert_eq!(m.get("bar").await.unwrap(), Some("barval".into()));
        m.commit().await.unwrap();
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        assert_eq!(m.get("bar").await.unwrap(), Some("barval".into()));
        let mut m_2 = m.map("nested");
        m_2.insert("baz", "bazval");
        m_2.stage().await.unwrap();
        assert_eq!(
            m.map("nested").get("baz").await.unwrap(),
            Some("bazval".into())
        );
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        assert_eq!(m.get("bar").await.unwrap(), Some("barval".into()));
        m_2.insert("bang", "bangval");
        m_2.stage().await.unwrap();
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        assert_eq!(m.get("bar").await.unwrap(), Some("barval".into()));
        assert_eq!(
            m.map("nested").get("baz").await.unwrap(),
            Some("bazval".into())
        );
        assert_eq!(
            m.map("nested").get("bang").await.unwrap(),
            Some("bangval".into())
        );
    }
    #[tokio::test]
    async fn isolated_nested_multi_value() {
        let f = Fixity::memory();
        let mut m = f.map(Path::new()).batch();
        m.insert("foo", "fooval");
        m.stage().await.unwrap();
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
        let mut m_2 = m.map("nested");
        m_2.insert("baz", "bazval");
        m_2.stage().await.unwrap();
        assert_eq!(
            m.map("nested").get("baz").await.unwrap(),
            Some("bazval".into())
        );
        assert_eq!(m.get("foo").await.unwrap(), Some("fooval".into()));
    }
}
