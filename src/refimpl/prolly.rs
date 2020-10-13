//! A [`prolly`] reference implementation.
use crate::{
    prolly::roller::{Config as RollerConfig, Roller},
    storage::StorageWrite,
    value::Value,
    Addr, Error,
};
#[allow(unused)]
pub struct Create<'s, S> {
    storage: &'s S,
    roller_config: RollerConfig,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self {
            storage,
            roller_config: RollerConfig::default(),
        }
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            // leaf: Leaf::new(storage, Roller::with_config(roller_config)),
        }
    }
}
impl<'s, S> Create<'s, S>
where
    S: StorageWrite,
{
    pub fn with_kvs(self, _kvs: Vec<(Value, Value)>) -> Result<Addr, Error> {
        // TODO: Make the Vec into a HashMap, to ensure uniqueness at this layer of the API.
        unimplemented!()
    }
}
struct Leaf<'s, S> {
    storage: &'s S,
    roller_config: RollerConfig,
    roller: Roller,
}
impl<'s, S> Leaf<'s, S> {
    pub fn new(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
        }
    }
}
impl<'s, S> Leaf<'s, S>
where
    S: StorageWrite,
{
    pub fn push(&mut self, kv: (Value, Value)) {
        // let boundary = roller.roll_bytes(&cjson::to_vec(&kv).map_err(|err| format!("{:?}", err))?);
        todo!()
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::storage::Memory};
    const DEFAULT_PATTERN: u32 = (1 << 8) - 1;
    #[test]
    fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        // let storage = Memory::new();
        // let mut tree =
        //     CreateTree::with_roller(&storage, RollerConfig::with_pattern(DEFAULT_PATTERN));
        // for (k, v) in (0..61).map(|i| (i, i * 10)) {
        //     tree = tree.push(k, v).unwrap();
        // }
        // dbg!(tree.flush());
        // dbg!(&storage);
    }
}
