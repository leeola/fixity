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
pub struct Create<'s, S> {
    leaf: Leaf<'s, S>,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
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
        self.leaf.flush().await
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
    pub async fn flush(&mut self) -> Result<Addr, Error> {
        if self.buffer.is_empty() {
            match self.parent.take() {
                // If there's no parent, this Leaf never hit a Boundary and thus this
                // Leaf itself is the root.
                //
                // This should be impossible.
                // A proper state machine would make this logic more safe, but async/await is
                // currently a bit immature for the design changes that would introduce.
                None => unreachable!("Create leaf missing parent and has empty buffer"),
                // If there is a parent, the root might be the parent, grandparent, etc.
                Some(mut parent) => parent.flush(None).await,
            }
        } else {
            let kvs = mem::replace(&mut self.buffer, Vec::new());
            let (node_key, node_addr, node_bytes) = {
                let node = Node::<_, Value, _>::Branch(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                (node.into_key_unchecked(), node_addr, node_bytes)
            };
            self.storage.write(node_addr.clone(), &*node_bytes).await?;
            match self.parent.take() {
                // If there's no parent, this Leaf never hit a Boundary and thus this
                // instance itself is the root.
                None => Ok(node_addr),
                // If there is a parent, the root might be the parent, grandparent, etc.
                Some(mut parent) => parent.flush(Some((dbg!(node_key), node_addr))).await,
            }
        }
    }
    pub async fn push(&mut self, kv: (Key, Value)) -> Result<(), Error> {
        // TODO: attempt to cache the serialized bytes for each kv pair into
        // a `Vec<[]byte,byte{}>` such that we can deserialize it into a `Vec<Value,Value>`.
        // *fingers crossed*. This requires the Read implementation up and running though.
        let boundary = self.roller.roll_bytes(&crate::value::serialize(&kv)?);
        dbg!(boundary);
        self.buffer.push(kv);
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
    /// Flush any partial kvs that have not yet found a boundary.
    ///
    /// The `kv` parameter is the child's address, either a Leaf or a Branch,
    /// who also flushed itself.
    ///
    /// # Returns
    /// A address for the root of the entire tree. Ie the Parent address for the
    /// tree.
    #[async_recursion::async_recursion]
    pub async fn flush(&mut self, kv: Option<(Key, Addr)>) -> Result<Addr, Error> {
        if let Some(kv) = kv {
            self.buffer.push(kv);
        }
        if self.buffer.is_empty() {
            match self.parent.take() {
                // If there's no parent, this Branch never hit a Boundary and thus this
                // instance itself is the root.
                //
                // This should be impossible.
                // A proper state machine would make this logic more safe, but async/await is
                // currently a bit immature for the design changes that would introduce.
                None => unreachable!("Create branch missing parent and has empty buffer"),
                // If there is a parent, the root might be the parent, grandparent, etc.
                Some(mut parent) => parent.flush(None).await,
            }
        } else {
            // self.buffer & self.parent "should" never be empty at the same time.
            let kvs = mem::replace(&mut self.buffer, Vec::new());
            let (node_key, node_addr, node_bytes) = {
                let node = Node::<_, Value, _>::Branch(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                (node.into_key_unchecked(), node_addr, node_bytes)
            };
            self.storage.write(node_addr.clone(), &*node_bytes).await?;
            match self.parent.take() {
                // If there's no parent, this Branch never hit a Boundary and thus this
                // instance itself is the root.
                None => Ok(node_addr),
                // If there is a parent, the root might be the parent, grandparent, etc.
                Some(mut parent) => parent.flush(Some((node_key, node_addr))).await,
            }
        }
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
