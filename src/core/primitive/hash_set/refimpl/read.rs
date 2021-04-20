use {
    crate::{
        core::{
            cache::{CacheRead, OwnedRef},
            deser::Deser,
            primitive::{
                hash_set::{HashKey, Node},
                prollytree::roller::{Config as RollerConfig, Roller},
            },
        },
        Addr, Error, Value,
    },
    std::{convert::TryInto, mem},
};
pub struct Read<'s, C> {
    cache: &'s C,
    root_addr: Addr,
}
impl<'s, C> Read<'s, C> {
    pub fn new(cache: &'s C, root_addr: Addr) -> Self {
        Self { cache, root_addr }
    }
    pub async fn to_vec(&self) -> Result<Vec<Value>, Error>
    where
        C: CacheRead,
    {
        self.recursive_to_vec(self.root_addr.clone()).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_to_vec(&self, addr: Addr) -> Result<Vec<Value>, Error>
    where
        C: CacheRead,
    {
        let owned_ref = self.cache.read_structured(&addr).await?;
        let node = owned_ref.into_owned_structured().try_into()?;
        match node {
            Node::Leaf(v) => Ok(v.into_iter().map(|(_, v)| v).collect::<Vec<_>>()),
            Node::Branch(v) => {
                let mut kvs = Vec::new();
                //for (_, addr) in v.as_slice() {
                for (_, addr) in v {
                    kvs.append(&mut self.recursive_to_vec(addr).await?);
                }
                Ok(kvs)
            },
        }
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::{super::Create, *},
        crate::core::Fixity,
        proptest::prelude::*,
        tokio::runtime::Runtime,
    };
    proptest! {
        #[test]
        fn read(
            values in prop::collection::vec(any::<Value>(), 1..5)
        ) {
            let mut env_builder = env_logger::builder();
            env_builder.is_test(true);
            if std::env::var("RUST_LOG").is_err() {
                env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
            }
            let _ = env_builder.try_init();
            Runtime::new().unwrap().block_on(async {
                read_impl(values).await
            });
        }
    }
    async fn read_impl(values: Vec<Value>) {
        let cache = Fixity::memory().into_cache();
        let tree = Create::with_roller(&cache, Deser::default(), RollerConfig::default());
        let addr = tree.with_vec(values.clone()).await.unwrap();
        let read_values = Read::new(&cache, addr).to_vec().await.unwrap();
        // read values likely won't be the same order, and will be deduplicated.
        // So rather than compare directly, we need to ensure there's no item missing
        // from the other, in either list.
        for read_value in read_values.iter() {
            assert!(values.contains(read_value));
        }
        for value in values.iter() {
            assert!(read_values.contains(value));
        }
    }
}
