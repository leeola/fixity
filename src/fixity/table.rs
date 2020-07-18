#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    super::{Scalar, VacantEntry, Value},
    crate::{
        storage::{Storage, StorageRead, StorageWrite},
        Addr, Error,
    },
    std::collections::HashMap,
};
pub struct Table<'s, S> {
    storage: &'s S,
}
impl<'s, S> Table<'s, S> {
    pub fn new(_storage: &S) -> Self {
        todo!("new table")
    }
    pub fn load(storage: &S, addr: Addr) -> Self {
        todo!("table load")
    }
}
#[cfg(test)]
pub mod test {
    /*
    use {
        super::*,
        crate::storage::{Memory, Storage, StorageRead, StorageWrite},
        maplit::hashmap,
    };
    #[test]
    fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        // let mut p = Prolly::new();
        Map::new(
            &storage,
            hashmap! {
                1 => 10,
                2 => 20,
            },
        )
        .unwrap();
        dbg!(&storage);
        let data = (0..20).map(|i| (i, i * 10)).collect::<HashMap<_, _>>();
        let m = Map::new(&storage, data);
        dbg!(&storage);
    }
    */
}
