use {
    crate::{
        deser::Deser,
        primitive::{
            prollylist::{Node, NodeOwned},
            prollytree::{
                roller::{Config as RollerConfig, Roller},
                ONE_LEN_BLOCK_WARNING,
            },
        },
        storage::StorageWrite,
        value::Value,
        Addr, Error,
    },
    std::mem,
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
    /// Constructs a prolly list based on the given `Value` pairs.
    ///
    /// # Errors
    ///
    /// Storage writes. No enforcement of sort order or uniqueness is enforced
    /// in Prolly Lists.
    pub async fn with_vec(mut self, items: Vec<Value>) -> Result<Addr, Error> {
        self.recursive_from_items(Node::Leaf(items)).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_from_items(&mut self, items: NodeOwned) -> Result<Addr, Error> {
        // All of the addrs produced from `items` blocks.
        let mut node_addrs = Vec::<Addr>::new();
        // A buffer for a block (branch or leaf) of items that have not found a boundary.
        let mut block_buf = match items {
            Node::Branch(_) => NodeOwned::Branch(Vec::new()),
            Node::Leaf(_) => NodeOwned::Leaf(Vec::new()),
        };
        for item in items.into_iter() {
            let boundary = self
                .roller
                .roll_bytes(&item.serialize_inner(&Deser::default())?);
            block_buf.push(item);
            if boundary {
                // Check for a case where a single key:value pair is equal to or exceeds
                // the bytes of an individual block. This typically indicates that the average
                // block size is too small or that the value being stored would be better
                // represented as a chunked byte array.
                let one_len_block = block_buf.len() == 1 && node_addrs.is_empty();
                if one_len_block {
                    log::warn!("{}", ONE_LEN_BLOCK_WARNING);
                }
                let node_addr = {
                    let node = {
                        let new_block_buf = match block_buf {
                            Node::Branch(_) => NodeOwned::Branch(Vec::new()),
                            Node::Leaf(_) => NodeOwned::Leaf(Vec::new()),
                        };
                        mem::replace(&mut block_buf, new_block_buf)
                    };
                    self.write_node(node).await?
                };
                node_addrs.push(node_addr);
            }
        }
        // if there are any remaining values in the buffer, no boundary was found for the
        // final block - so write them together as the last block in the series.
        if !block_buf.is_empty() {
            let node_addr = self.write_node(block_buf).await?;
            node_addrs.push(node_addr);
        }
        if node_addrs.len() == 1 {
            Ok(node_addrs.pop().expect("node_addrs impossibly missing"))
        } else {
            self.recursive_from_items(Node::Branch(node_addrs)).await
        }
    }
    async fn write_node(&self, node: NodeOwned) -> Result<Addr, Error> {
        let node_bytes = Deser::default().to_vec(&node)?;
        let node_addr = Addr::hash(&node_bytes);
        self.storage.write(node_addr.clone(), &*node_bytes).await?;
        Ok(node_addr)
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
        let contents = vec![(0u32..20), (0..200), (0..2_000)];
        for content in contents {
            let content = content.map(Value::from).collect::<Vec<_>>();
            let storage = Memory::new();
            let tree = Create::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
            let addr = tree.with_vec(content).await.unwrap();
            dbg!(addr);
        }
    }
}
