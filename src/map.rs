use {
    crate::{
        core::{
            self,
            cache::{CacheRead, CacheWrite},
            misc::range_ext::{OwnedRangeBounds, RangeBoundsExt},
            primitive::prollytree::refimpl,
            workspace::Workspace,
            Commit,
        },
        error::Type as TypeError,
        path::{SegmentResolve, SegmentUpdate},
        Addr, Error, Key, Value,
    },
    std::fmt,
};
pub struct Map<'f>(Box<dyn InnerMap<'f> + 'f>);
impl<'f> Map<'f> {
    pub fn new<C, W>(inner: core::Map<'f, C, W>) -> Self
    where
        C: CacheRead + CacheWrite,
        W: Workspace,
    {
        Self(Box::new(inner))
    }
    pub fn batch(&self) -> BatchMap<'f> {
        self.0.inner_batch()
    }
    /// Insert a value into the map to later be committed.
    ///
    /// This value is written to the store immediately, as a staged value.
    /// For a large multi-key insertion, see [`Self::batch`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use fixity::{Fixity, Path, Addr};
    /// let f = Fixity::memory();
    /// let mut m = f.map(Path::new());
    /// let _: Addr = m.insert("foo", "bar").await.unwrap();
    /// assert_eq!(m.get("foo").await.unwrap(), Some("bar".into()));
    /// # }
    /// ```
    pub async fn insert<K, V>(&mut self, key: K, value: V) -> Result<Addr, Error>
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        self.0.inner_insert(key.into(), value.into()).await
    }
    /// Get a value at the current [`Path`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use fixity::{Fixity, Path, Addr};
    /// let f = Fixity::memory();
    /// let mut m = f.map(Path::new());
    /// let _: Addr = m.insert("foo", "bar").await.unwrap();
    /// assert_eq!(m.get("foo").await.unwrap(), Some("bar".into()));
    /// # }
    /// ```
    pub async fn get<K>(&self, key: K) -> Result<Option<Value>, Error>
    where
        K: Into<Key>,
    {
        self.0.inner_get(key.into()).await
    }
    pub async fn iter<R>(
        &self,
        range: R,
    ) -> Result<Box<dyn Iterator<Item = Result<(Key, Value), Error>>>, Error>
    where
        R: RangeBoundsExt<Key>,
    {
        self.0.inner_iter(range.into_bounds()).await
    }
    pub async fn commit(&self) -> Result<Addr, Error> {
        self.0.inner_commit().await
    }
}
#[async_trait::async_trait]
trait InnerMap<'f> {
    fn inner_batch(&self) -> BatchMap<'f>;
    async fn inner_insert(&self, key: Key, value: Value) -> Result<Addr, Error>;
    async fn inner_get(&self, key: Key) -> Result<Option<Value>, Error>;
    async fn inner_iter(
        &self,
        range: OwnedRangeBounds<Key>,
    ) -> Result<Box<dyn Iterator<Item = Result<(Key, Value), Error>>>, Error>;
    // async fn inner_remove(&self, key: Key) -> Result<Addr, Error>;
    async fn inner_commit(&self) -> Result<Addr, Error>;
}
#[async_trait::async_trait]
impl<'f, C, W> InnerMap<'f> for core::Map<'f, C, W>
where
    C: CacheRead + CacheWrite,
    W: Workspace,
{
    fn inner_batch(&self) -> BatchMap<'f> {
        let b = self.batch();
        BatchMap::new(b)
    }
    async fn inner_insert(&self, key: Key, value: Value) -> Result<Addr, Error> {
        self.insert(key, value).await
    }
    async fn inner_get(&self, key: Key) -> Result<Option<Value>, Error> {
        self.get(key).await
    }
    async fn inner_iter(
        &self,
        range: OwnedRangeBounds<Key>,
    ) -> Result<Box<dyn Iterator<Item = Result<(Key, Value), Error>>>, Error> {
        self.iter(range).await
    }
    // async fn inner_remove(&self, key: Key) -> Result<Addr, Error> {
    //     self.remove(key).await
    // }
    async fn inner_commit(&self) -> Result<Addr, Error> {
        self.commit().await
    }
}
pub struct BatchMap<'f>(Box<dyn InnerBatchMap + 'f>);
impl<'f> BatchMap<'f> {
    pub fn new<C, W>(inner_map: core::map::BatchMap<'f, C, W>) -> Self
    where
        C: CacheRead + CacheWrite,
        W: Workspace,
    {
        Self(Box::new(inner_map))
    }
    /// Drop the internal change cache that has not yet been staged or committed to storage.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map,path::Path};
    /// let f = Fixity::memory();
    /// let mut m = f.map(Path::new()).batch();
    /// m.insert("foo", "bar");
    /// m.clear();
    /// assert!(m.get("foo").await.unwrap().is_none());
    /// # }
    /// ```
    pub fn clear(&mut self) {
        self.0.inner_clear()
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
    /// # use fixity::{Fixity,Map,path::Path};
    /// let f = Fixity::memory();
    /// let mut m_1 = f.map(Path::new()).batch();
    /// let m_2 = f.map(Path::new()).batch();
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
        self.0.inner_insert(key.into(), value.into())
    }
    pub async fn get<K>(&self, key: K) -> Result<Option<Value>, Error>
    where
        K: Into<Key>,
    {
        self.0.inner_get(key.into()).await
    }
    /// Write any changes to storage, staging them for a later commit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map,path::Path};
    /// let f = Fixity::memory();
    /// let mut m_1 = f.map(Path::new()).batch();
    /// let m_2 = f.map(Path::new()).batch();
    /// m_1.insert("foo", "bar");
    /// // not yet written to storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), None);
    /// let _staged_addr = m_1.stage().await.unwrap();
    /// // now in storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), Some("bar".into()));
    /// # }
    pub async fn stage(&mut self) -> Result<Addr, Error> {
        self.0.inner_stage().await
    }
    /// Write any [staged](Self::stage) changes at the current [`Path`] into the workspace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use fixity::{Fixity,Map,path::Path};
    /// let f = fixity::fixity::Fixity::memory();
    /// let mut m_1 = f.map(Path::new()).batch();
    /// let m_2 = f.map(Path::new()).batch();
    /// m_1.insert("foo", "bar");
    /// // not yet written to storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), None);
    /// m_1.stage().await.unwrap();
    /// m_1.commit().await.unwrap();
    /// // now in storage.
    /// assert_eq!(m_2.get("foo").await.unwrap(), Some("bar".into()));
    /// # }
    pub async fn commit(&mut self) -> Result<Addr, Error> {
        self.0.inner_commit().await
    }
}
#[async_trait::async_trait]
trait InnerBatchMap {
    fn inner_clear(&mut self);
    fn inner_insert(&mut self, key: Key, value: Value);
    async fn inner_get(&self, key: Key) -> Result<Option<Value>, Error>;
    async fn inner_stage(&mut self) -> Result<Addr, Error>;
    async fn inner_commit(&mut self) -> Result<Addr, Error>;
}
#[async_trait::async_trait]
impl<'f, C, W> InnerBatchMap for core::map::BatchMap<'f, C, W>
where
    C: CacheRead + CacheWrite,
    W: Workspace,
{
    fn inner_clear(&mut self) {
        self.clear();
    }
    fn inner_insert(&mut self, key: Key, value: Value) {
        self.insert(key, value);
    }
    async fn inner_get(&self, key: Key) -> Result<Option<Value>, Error> {
        self.get(key).await
    }
    async fn inner_stage(&mut self) -> Result<Addr, Error> {
        self.stage().await
    }
    async fn inner_commit(&mut self) -> Result<Addr, Error> {
        self.commit().await
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathSegment {
    pub key: Key,
}
impl PathSegment {
    pub fn new<T: Into<Key>>(t: T) -> Self {
        Self { key: t.into() }
    }
}
impl fmt::Debug for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Map(")?;
        self.key.fmt(f)?;
        f.write_str(")")
    }
}
impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Map(")?;
        self.key.fmt(f)?;
        f.write_str(")")
    }
}
#[async_trait::async_trait]
impl<C> SegmentResolve<C> for PathSegment
where
    C: CacheRead,
{
    async fn resolve(&self, storage: &C, self_addr: Addr) -> Result<Option<Addr>, Error> {
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
            },
        };
        Ok(Some(addr))
    }
}
#[async_trait::async_trait]
impl<C> SegmentUpdate<C> for PathSegment
where
    C: CacheRead + CacheWrite,
{
    async fn update(
        &self,
        storage: &C,
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
impl<T> From<T> for PathSegment
where
    T: Into<Key>,
{
    fn from(t: T) -> Self {
        Self { key: t.into() }
    }
}
