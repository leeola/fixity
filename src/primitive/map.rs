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
pub struct Map<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
    reader: Option<refimpl::Read<'s, S>>,
    stage: HashMap<Key, Value>,
}
impl<'s, S> Map<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        let reader = addr
            .as_ref()
            .map(|addr| refimpl::Read::new(storage, addr.clone()));
        Self {
            storage,
            addr,
            reader,
            stage: HashMap::new(),
        }
    }
    pub fn build(storage: &'s S) -> Builder<'s, S> {
        Builder::new(storage)
    }
    pub fn insert<K, V>(&mut self, k: K, v: V) -> Option<Value>
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        self.stage.insert(k.into(), v.into())
    }
    pub fn append<I, K, V>(&mut self, i: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Key>,
        V: Into<Value>,
    {
        i.into_iter().for_each(|(k, v)| {
            self.insert(k.into(), v.into());
        });
    }
}
impl<'s, S> Map<'s, S>
where
    S: StorageRead,
{
    pub async fn get<K>(&self, k: K) -> Result<Option<Value>, Error>
    where
        K: Into<Key>,
    {
        let k = k.into();
        if let Some(v) = self.stage.get(&k) {
            return Ok(Some(v.clone()));
        }
        let reader = match &self.reader {
            Some(r) => r,
            None => return Ok(None),
        };
        reader.get(&k).await
    }
}
#[async_trait::async_trait]
impl<'s, S> InsertAddr for Map<'s, S>
where
    S: Sync,
{
    async fn insert_addr(&mut self, key: Key, addr: Addr) -> Result<(), Error> {
        self.insert(key, Value::from(addr));
        Ok(())
    }
}
#[async_trait::async_trait]
impl<'s, S> GetAddr for Map<'s, S>
where
    S: StorageRead,
{
    async fn get_addr(&self, key: Key) -> Result<Option<Addr>, Error> {
        match self.get(key).await? {
            Some(Value::Addr(addr)) => Ok(Some(addr)),
            None => Ok(None),
            Some(_) => Err(TypeError::UnexpectedValueVariant {
                at_key: None,
                at_addr: self.addr.clone(),
            }
            .into()),
        }
    }
}
#[async_trait::async_trait]
impl<'s, S> Flush for Map<'s, S>
where
    S: StorageRead + StorageWrite,
{
    async fn flush(&mut self) -> Result<Addr, Error> {
        let kvs = mem::replace(&mut self.stage, HashMap::new()).into_iter();
        if let Some(addr) = self.addr.as_ref() {
            // TODO: these should probably be converted to changes in
            // the `Map` itself. Quick and dirty for the moment though.
            let kvs = kvs
                .map(|(k, v)| (k, refimpl::Change::Insert(v)))
                .collect::<Vec<_>>();
            refimpl::Update::new(self.storage, addr.clone())
                .with_vec(kvs)
                .await
        } else {
            let kvs = kvs.collect::<Vec<_>>();
            refimpl::Create::new(self.storage).with_vec(kvs).await
        }
    }
}
pub struct Builder<'s, S> {
    storage: &'s S,
}
impl<'s, S> Builder<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self { storage }
    }
}
#[async_trait::async_trait]
impl<'s, S> Build for Builder<'s, S>
where
    S: StorageRead + StorageWrite,
{
    type Primitive = Map<'s, S>;
    async fn build(self, addr: Option<Addr>) -> Result<Self::Primitive, Error> {
        Ok(Map::new(self.storage, addr))
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::storage::Memory};
    #[test]
    fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let mut m = Map::new(&storage, None);
        m.append((0..20).map(|i| (i, i * 10)));
        // dbg!(&storage);
    }
    /*
    #[test]
    fn equality() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let datas = vec![
            hashmap! {
                1 => 10,
                2 => 20,
            },
            (0..20).map(|i| (i, i * 10)).collect::<HashMap<_, _>>(),
        ];
        for data in datas {
            let storage_a = Memory::new();
            Map::new(&storage_a, data.clone()).unwrap();
            let storage_b = Memory::new();
            Map::new(&storage_b, data).unwrap();
            assert_eq!(storage_a, storage_b);
        }
    }
    */
}
