//! A [`prolly`] reference implementation.
use crate::{storage::StorageWrite, value::Value, Addr, Error};
#[allow(unused)]
pub struct Create<'s, S> {
    storage: &'s S,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self { storage }
    }
}
impl<'s, S> Create<'s, S>
where
    S: StorageWrite,
{
    pub fn from_kvs(storage: &'s S, _kvs: Vec<(Value, Value)>) -> Result<Addr, Error> {
        let _create = Self::new(storage);
        unimplemented!()
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
