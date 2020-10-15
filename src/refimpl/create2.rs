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
    // branches: Vec<Branch<'s, S>>,
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
