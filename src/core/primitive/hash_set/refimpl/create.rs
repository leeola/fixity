use {
    crate::{
        core::{
            cache::CacheWrite,
            deser::Deser,
            primitive::{
                hash_set::{HashKey, Node},
                prollytree::roller::{Config as RollerConfig, Roller},
            },
        },
        Addr, Error, Value,
    },
    std::mem,
};
pub struct Create<'s, C> {
    cache: &'s C,
    deser: Deser,
    roller: Roller,
}
impl<'s, C> Create<'s, C> {
    pub fn new(cache: &'s C, deser: Deser) -> Self {
        Self::with_roller(cache, deser, RollerConfig::default())
    }
    pub fn with_roller(cache: &'s C, deser: Deser, roller_config: RollerConfig) -> Self {
        Self {
            cache,
            deser,
            roller: Roller::with_config(roller_config),
        }
    }
    /// Constructs a prolly list based on the given `Value` pairs.
    ///
    /// # Errors
    ///
    /// Cache writes. No enforcement of sort order or uniqueness is enforced
    /// in Prolly Lists.
    pub async fn with_vec(mut self, items: Vec<Value>) -> Result<Addr, Error>
    where
        C: CacheWrite,
    {
        let mut kvs = items
            .into_iter()
            .map(|value| {
                let b = self.deser.to_vec(&value)?;
                Ok((HashKey::hash(b), value))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        // Ensure the kvs are sorted - as the trees require sorting.
        // unstable should be fine, since the keys will (soon) be unique.
        kvs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        // Dropping identical items is fine, but perhaps a warning should be
        // provided as it's a waste of CPU.
        kvs.dedup_by(|a, b| a.0 == b.0);
        self.recursive_from_items(Node::Leaf(kvs)).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_from_items(&mut self, items: Node) -> Result<Addr, Error>
    where
        C: CacheWrite,
    {
        // All of the addrs produced from `items` blocks.
        let mut node_addrs = Vec::<(HashKey, Addr)>::new();
        // A buffer for a block (branch or leaf) of items that have not found a boundary.
        let mut block_buf = match items {
            Node::Branch(_) => Node::Branch(Vec::new()),
            Node::Leaf(_) => Node::Leaf(Vec::new()),
        };
        for item in items.into_iter() {
            let boundary = self.roller.roll_bytes(&item.serialize_inner(&self.deser)?);
            block_buf.push(item);
            if boundary {
                let node_kv = {
                    let node = {
                        let new_block_buf = match block_buf {
                            Node::Branch(_) => Node::Branch(Vec::new()),
                            Node::Leaf(_) => Node::Leaf(Vec::new()),
                        };
                        mem::replace(&mut block_buf, new_block_buf)
                    };
                    self.write_node(node).await?
                };
                node_addrs.push(node_kv);
            }
        }
        // if there are any remaining values in the buffer, no boundary was found for the
        // final block - so write them together as the last block in the series.
        if !block_buf.is_empty() {
            let node_kv = self.write_node(block_buf).await?;
            node_addrs.push(node_kv);
        }
        if node_addrs.len() == 1 {
            Ok(node_addrs.pop().expect("node_addrs impossibly missing").1)
        } else {
            self.recursive_from_items(Node::Branch(node_addrs)).await
        }
    }
    async fn write_node(&self, node: Node) -> Result<(HashKey, Addr), Error>
    where
        C: CacheWrite,
    {
        let node_key = node
            .first_key()
            .expect("first key impossibly missing")
            .clone();
        let node_addr = self.cache.write_structured(node).await?;
        Ok((node_key, node_addr))
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::core::Fixity, proptest::prelude::*, tokio::runtime::Runtime};
    #[tokio::test]
    async fn single_value() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let cache = Fixity::memory().into_cache();
        let tree = Create::with_roller(&cache, Deser::default(), RollerConfig::default());
        let addr = tree.with_vec(vec![Value::from(1)]).await.unwrap();
        dbg!(addr);
    }
    proptest! {
        #[test]
        fn inputs(
            values in prop::collection::vec(Addr::prop().prop_map(Value::from), 1..100)
        ) {
            let mut env_builder = env_logger::builder();
            env_builder.is_test(true);
            if std::env::var("RUST_LOG").is_err() {
                env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
            }
            let _ = env_builder.try_init();
            Runtime::new().unwrap().block_on(async {
                test_inputs(values).await
            });
        }
    }
    async fn test_inputs(values: Vec<Value>) {
        let cache = Fixity::memory().into_cache();
        let tree = Create::with_roller(&cache, Deser::default(), RollerConfig::default());
        let _addr = tree.with_vec(values).await.unwrap();
    }
}
