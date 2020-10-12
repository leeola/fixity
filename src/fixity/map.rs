// storage::{StorageRead, StorageWrite},
use crate::Addr;
#[allow(unused)]
pub struct Map<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
}
impl<'s, S> Map<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self { storage, addr }
    }
}
impl<'s, S> Map<'s, S> {
    pub fn insert<K, V>(&mut self, k: K, v: V) {
        unimplemented!("insert")
    }
    pub fn append<K, V, I>(&mut self, into_iter: IntoIter<Item = (K, V)>) {
        unimplemented!("append")
    }
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
        let _storage = Memory::new();
        // let mut p = Prolly::new();
        // Map::new(
        //     &storage,
        //     hashmap! {
        //         1 => 10,
        //         2 => 20,
        //     },
        // )
        // .unwrap();
        // dbg!(&storage);
        // let data = (0..20).map(|i| (i, i * 10)).collect::<HashMap<_, _>>();
        // let m = Map::new(&storage, data);
        // dbg!(&storage);
    }
    /*
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
    */
}
