//! A [`prolly`] reference implementation.
use {
    crate::{
        prolly::{
            node::Node,
            roller::{Config as RollerConfig, Roller},
        },
        storage::StorageWrite,
        value::{Key, Value},
        Addr, Error,
    },
    std::mem,
};
#[allow(unused)]
pub struct Create<'s, S> {
    storage: &'s S,
    roller_config: RollerConfig,
    leaf: Leaf<'s, S>,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config: roller_config.clone(),
            leaf: Leaf::new(storage, roller_config),
        }
    }
}
impl<'s, S> Create<'s, S>
where
    S: StorageWrite,
{
    pub async fn with_kvs(mut self, mut kvs: Vec<(Key, Value)>) -> Result<Addr, Error> {
        // TODO: Make the Vec into a HashMap, to ensure uniqueness at this layer of the API.

        // unstable should be fine, since the incoming values are unique.
        kvs.sort_unstable();
        for kv in kvs.into_iter() {
            self.leaf.push(kv).await?;
        }
        todo!("Create::with kvs")
    }
}
struct Leaf<'s, S> {
    storage: &'s S,
    roller_config: RollerConfig,
    roller: Roller,
    buffer: Vec<(Key, Value)>,
    parent: Option<Branch<'s, S>>,
}
impl<'s, S> Leaf<'s, S> {
    pub fn new(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            buffer: Vec::new(),
            parent: None,
        }
    }
}
impl<'s, S> Leaf<'s, S>
where
    S: StorageWrite,
{
    pub async fn push(&mut self, kv: (Key, Value)) -> Result<(), Error> {
        // TODO: attempt to cache the serialized bytes for each kv pair into
        // a `Vec<[]byte,byte{}>` such that we can deserialize it into a `Vec<Value,Value>`.
        // *fingers crossed*. This requires the Read implementation up and running though.
        let boundary = self.roller.roll_bytes(&crate::value::serialize(&kv)?);
        self.buffer.push(kv);
        dbg!(boundary);
        if boundary {
            let is_first_kv = self.buffer.is_empty() && self.parent.is_none();
            if is_first_kv {
                log::warn!(
                    "writing key & value that exceeds block size, this is highly inefficient"
                );
            }
            let (node_key, node_addr) = {
                let kvs = mem::replace(&mut self.buffer, Vec::new());
                let node = Node::<_, _, Addr>::Leaf(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            self.parent
                .get_or_insert_with(|| Branch::new(storage, roller_config.clone()))
                .push((node_key, node_addr.into()))
                .await?;
        }
        Ok(())
    }
}
struct Branch<'s, S> {
    storage: &'s S,
    roller_config: RollerConfig,
    roller: Roller,
    buffer: Vec<(Key, Addr)>,
    parent: Option<Box<Branch<'s, S>>>,
}
impl<'s, S> Branch<'s, S> {
    pub fn new(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            buffer: Vec::new(),
            parent: None,
        }
    }
}
impl<'s, S> Branch<'s, S>
where
    S: StorageWrite,
{
    #[async_recursion::async_recursion]
    pub async fn flush(&mut self) -> Result<(Key, Addr), Error> {
        if let Some(mut parent) = self.parent.take() {
            let ka = parent.flush().await?;
            self.buffer.push(ka);
        }
        let kvs = mem::replace(&mut self.buffer, Vec::new());
        let node = Node::<_, Value, _>::Branch(kvs);
        let (node_addr, node_bytes) = node.as_bytes()?;
        self.storage.write(node_addr.clone(), &*node_bytes).await?;
        Ok((node.into_key_unchecked(), node_addr))
    }
    #[async_recursion::async_recursion]
    pub async fn push(&mut self, kv: (Key, Addr)) -> Result<(), Error> {
        // TODO: attempt to cache the serialized bytes for each kv pair into
        // a `Vec<[]byte,byte{}>` such that we can deserialize it into a `Vec<Value,Value>`.
        // *fingers crossed*. This requires the Read implementation up and running though.
        let boundary = self.roller.roll_bytes(&crate::value::serialize(&kv)?);
        dbg!(&kv.0, boundary, self.buffer.len());
        self.buffer.push(kv);
        if boundary {
            let first_kv = self.buffer.is_empty() && self.parent.is_none();
            if first_kv {
                log::warn!(
                    "writing key & value that exceeds block size, this is highly inefficient"
                );
            }
            let (node_key, node_addr) = {
                let kvs = mem::replace(&mut self.buffer, Vec::new());
                let node = Node::<_, Value, _>::Branch(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            self.parent
                .get_or_insert_with(|| Box::new(Branch::new(storage, roller_config.clone())))
                .push((node_key, node_addr.into()))
                .await?;
        }
        Ok(())
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::storage::Memory};
    /// A smaller value to use with the roller, producing smaller average block sizes.
    const TEST_PATTERN: u32 = (1 << 8) - 1;
    #[tokio::test]
    async fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let tree = Create::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
        let kvs = (0..400)
            .map(|i| (i, i * 10))
            .map(|(k, v)| (Key::from(k), Value::from(v)))
            .collect::<Vec<_>>();
        let addr = tree.with_kvs(kvs).await.unwrap();
        dbg!(addr);
        dbg!(&storage);
        // dbg!(tree.flush());
        // dbg!(&storage);
    }
}
