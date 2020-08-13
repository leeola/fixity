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
pub struct Map<'s, S> {
    storage: &'s S,
    // items: Vec<NodeItem<K, V>>,
}
impl<'s, S> Map<'s, S> {
    pub fn new(_storage: &S) -> Self {
        todo!("new map")
    }
    // pub fn load<S>(storage: &S, map_ref: Ref) -> Self
    // where
    //     S: StorageWrite,
    // {
    //     todo!("map load")
    // }
}
#[cfg(test)]
pub mod test {
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
    #[test]
    fn equality() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let datas = vec![
            hashmap! {
                1 => 10,
                2 => 20,
            },
            (0..20).map(|i| (i, i * 10)).collect::<HashMap<_, _>>(),
        ];
        for data in datas {
            let storage_a = Memory::new();
            Map::new(&storage_a, data.clone()).unwrap();
            let storage_b = Memory::new();
            Map::new(&storage_b, data).unwrap();
            assert_eq!(storage_a, storage_b);
        }
    }
}
