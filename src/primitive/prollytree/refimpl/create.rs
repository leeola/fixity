use {
    crate::{
        deser::{Deser, Error as DeserError},
        primitive::prollytree::{
            roller::{Config as RollerConfig, Roller},
            Node, NodeOwned,
        },
        storage::StorageWrite,
        value::{Key, Value},
        Addr, Error,
    },
    std::{collections::HashMap, mem},
};
pub struct Create<'s, S> {
    storage: &'s S,
    roller: Roller,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller: Roller::with_config(roller_config),
        }
    }
}
impl<'s, S> Create<'s, S>
where
    S: StorageWrite,
{
    /// Constructs a prolly tree based on the given `Key, Value` pairs.
    ///
    /// # Errors
    ///
    /// If the provided vec contains non-unique keys or any writes to storage fail
    /// an error is returned.
    pub async fn with_vec(mut self, mut kvs: Vec<(Key, Value)>) -> Result<Addr, Error> {
        // Ensure the kvs are sorted - as the trees require sorting.
        // unstable should be fine, since the keys will (soon) be unique.
        kvs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        // Ensure the kvs are unique.
        let maybe_dups_len = kvs.len();
        kvs.dedup_by(|a, b| a.0 == b.0);
        if maybe_dups_len != kvs.len() {
            return Err(Error::Prolly {
                message: "cannot construct prolly tree with non-unique keys".to_owned(),
            });
        }
        self.recursive_from_kvs(KeyValues::from(kvs)).await
    }
    /// Constructs a prolly tree based on the given `Key, Value` pairs.
    ///
    /// # Errors
    ///
    /// If the provided vec contains non-unique keys or any writes to storage fail
    /// an error is returned.
    pub async fn with_hashmap(mut self, kvs: HashMap<Key, Value>) -> Result<Addr, Error> {
        let mut kvs = kvs.into_iter().collect::<Vec<_>>();
        // Ensure the kvs are sorted - as the trees require sorting.
        // unstable should be fine, since the keys will (soon) be unique.
        kvs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        self.recursive_from_kvs(KeyValues::from(kvs)).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_from_kvs(&mut self, kvs: KeyValues) -> Result<Addr, Error> {
        let mut block_addrs = Vec::new();
        let mut block_buf = KeyValues::new_of_same_variant(&kvs);
        for kv in kvs.into_iter() {
            let boundary = self
                .roller
                .roll_bytes(&kv.serialize_inner(&Deser::default())?);
            block_buf.push(kv);
            if boundary {
                // Check for a case where a single key:value pair is equal to or exceeds
                // the bytes of an individual block. This typically indicates that the average
                // block size is too small or that the value being stored would be better
                // represented as a chunked byte array.
                let one_len_block = block_buf.len() == 1 && block_addrs.is_empty();
                if one_len_block {
                    log::warn!(
                        "writing key & value that exceeds block size, this is highly inefficient"
                    );
                }
                let (block_key, block_addr) = {
                    let new_block_buf = KeyValues::new_of_same_variant(&block_buf);
                    self.write_block(mem::replace(&mut block_buf, new_block_buf))
                        .await?
                };
                block_addrs.push((block_key, block_addr));
            }
        }
        // if there are any remaining key:value pairs, no boundary was found for the
        // final block - so write them together as the last block in the series.
        if !block_buf.is_empty() {
            let (block_key, block_addr) = {
                let new_block_buf = KeyValues::new_of_same_variant(&block_buf);
                self.write_block(mem::replace(&mut block_buf, new_block_buf))
                    .await?
            };
            block_addrs.push((block_key, block_addr));
        }
        if block_addrs.len() == 1 {
            Ok(block_addrs.pop().expect("key:addr impossibly missing").1)
        } else {
            self.recursive_from_kvs(KeyValues::from(block_addrs)).await
        }
    }
    async fn write_block(&self, block_buf: KeyValues) -> Result<(Key, Addr), Error> {
        let key = block_buf
            .first_key()
            .expect("first key impossibly missing")
            .clone();
        let node = NodeOwned::from(block_buf);
        let (node_addr, node_bytes) = node.as_bytes()?;
        self.storage.write(node_addr.clone(), &*node_bytes).await?;
        Ok((key, node_addr))
    }
}
enum KeyValues {
    KeyValues(Vec<(Key, Value)>),
    KeyAddrs(Vec<(Key, Addr)>),
}
impl KeyValues {
    pub fn new_of_same_variant(same_as: &KeyValues) -> Self {
        match same_as {
            Self::KeyValues(_) => Self::KeyValues(Vec::new()),
            Self::KeyAddrs(_) => Self::KeyAddrs(Vec::new()),
        }
    }
    pub fn first_key(&self) -> Option<&Key> {
        match self {
            Self::KeyValues(v) => v.first().map(|(k, _)| k),
            Self::KeyAddrs(v) => v.first().map(|(k, _)| k),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::KeyValues(v) => v.is_empty(),
            Self::KeyAddrs(v) => v.is_empty(),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::KeyValues(v) => v.len(),
            Self::KeyAddrs(v) => v.len(),
        }
    }
    /// Push the given `KeyValue` into the `KeyValues`.
    ///
    /// # Panics
    ///
    /// If the variants are not aligned between this instance and what is being pushed
    /// this code will panic.
    pub fn push(&mut self, kv: KeyValue) {
        match (self, kv) {
            (Self::KeyValues(ref mut v), KeyValue::KeyValue(kv)) => v.push(kv),
            (Self::KeyAddrs(ref mut v), KeyValue::KeyAddr(kv)) => v.push(kv),
            (_, _) => panic!("KeyValue pushed to unaligned KeyValues enum vec"),
        }
    }
    pub fn into_iter(self) -> Box<dyn Iterator<Item = KeyValue> + Send> {
        match self {
            Self::KeyValues(v) => Box::new(v.into_iter().map(KeyValue::from)),
            Self::KeyAddrs(v) => Box::new(v.into_iter().map(KeyValue::from)),
        }
    }
}
impl From<Vec<(Key, Value)>> for KeyValues {
    fn from(kvs: Vec<(Key, Value)>) -> Self {
        Self::KeyValues(kvs)
    }
}
impl From<Vec<(Key, Addr)>> for KeyValues {
    fn from(kvs: Vec<(Key, Addr)>) -> Self {
        Self::KeyAddrs(kvs)
    }
}
impl From<KeyValues> for NodeOwned {
    fn from(kvs: KeyValues) -> Self {
        match kvs {
            KeyValues::KeyValues(v) => Node::Leaf(v),
            KeyValues::KeyAddrs(v) => Node::Branch(v),
        }
    }
}
enum KeyValue {
    KeyValue((Key, Value)),
    KeyAddr((Key, Addr)),
}
impl KeyValue {
    pub fn serialize_inner(&self, deser: &Deser) -> Result<Vec<u8>, DeserError> {
        match self {
            Self::KeyValue(kv) => deser.serialize(kv),
            Self::KeyAddr(kv) => deser.serialize(kv),
        }
    }
}
impl From<(Key, Value)> for KeyValue {
    fn from(kv: (Key, Value)) -> Self {
        Self::KeyValue(kv)
    }
}
impl From<(Key, Addr)> for KeyValue {
    fn from(kv: (Key, Addr)) -> Self {
        Self::KeyAddr(kv)
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::storage::Memory};
    /// A smaller value to use with the roller, producing smaller average block sizes.
    const TEST_PATTERN: u32 = (1 << 8) - 1;
    #[tokio::test]
    async fn create_without_failure() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let contents = vec![(0..20), (0..200), (0..2_000)];
        for content in contents {
            let content = content
                .map(|i| (i, i * 10))
                .map(|(k, v)| (Key::from(k), Value::from(v)))
                .collect::<Vec<_>>();
            let storage = Memory::new();
            let tree = Create::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
            let addr = tree.with_vec(content).await.unwrap();
            dbg!(addr);
        }
    }
}
