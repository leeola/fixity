use {
    crate::{
        prolly::{
            node::NodeOwned,
            roller::{Config as RollerConfig, Roller},
        },
        storage::StorageWrite,
        value::{Key, Value},
        Addr, Error,
    },
    std::mem,
};
enum Kvs {
    KeyValues(Vec<(Key, Value)>),
    KeyAddrs(Vec<(Key, Addr)>),
}
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
    pub async fn from_vec(mut self, mut kvs: Vec<(Key, Value)>) -> Result<Addr, Error> {
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
self.recursive_from_kvs(kvs.into()).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_from_kvs(mut self, mut kvs: Kvs) -> Result<Addr, Error> {
        let mut block_addrs = Vec::new();
        let mut block_buf = Kvs::new();
        for kv in kvs.into_iter() {
            let boundary = self.roller.roll_bytes(&kv.serialize_inner());
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
                let block_key = block_buf
                    .first_key()
                    .expect("buffer impossibly empty")
                    .clone();
                let node = mem::replace(&mut block_buf, Kvs::new()).into_node();
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                block_addrs.push((block_key, node_addr));
            }
        }
        // if there are any remaining key:value pairs, no boundary was found for the
        // final block - so write them together as the last block in the series.
        if !block_buf.is_empty() {
            let block_key = block_buf
                .first()
                .expect("first key impossibly missing")
                .0
                .clone();
            let node = mem::replace(&mut block_buf, Kvs::new()).into_node();
            let (node_addr, node_bytes) = node.as_bytes()?;
            self.storage.write(node_addr.clone(), &*node_bytes).await?;
            let block_kv = self.write_block(mem::replace(&mut block_buf, Kvs::new())
            block_addrs.push((block_key, node_addr));
        }
        if block_addrs.len() == 1 {
            Ok(block_addrs.pop().expect("key:addr impossibly missing").1)
        } else {
            self.recursive_from_kvs(block_addrs.into()).await
        }
    }
    async fn write_block(&self, block_buf: Kvs) -> Result<(Key, Addr), Error> {
            let key = block_buf
                .first()
                .expect("first key impossibly missing")
                .0
                .clone();
            let node = block_buf.into_node();
            let (node_addr, node_bytes) = node.as_bytes()?;
            self.storage.write(node_addr.clone(), &*node_bytes).await?;
            Ok((key, node_addr))
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
            let addr = tree.from_vec(content).await.unwrap();
            dbg!(addr);
        }
    }
}
