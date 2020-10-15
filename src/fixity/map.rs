use {
    crate::storage::{StorageRead, StorageWrite},
    crate::{
        refimpl::prolly,
        value::{Key, Value},
        Addr, Error,
    },
    std::{collections::HashMap, mem},
};
pub struct Map<'s, S> {
    storage: &'s S,
    addr: Option<Addr>,
    stage: HashMap<Key, Value>,
}
impl<'s, S> Map<'s, S> {
    pub fn new(storage: &'s S, addr: Option<Addr>) -> Self {
        Self {
            storage,
            addr,
            stage: HashMap::new(),
        }
    }
    pub fn insert<K, V>(&mut self, k: K, v: V) -> Option<Value>
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        self.stage.insert(k.into(), v.into())
    }
    pub fn append<I, K, V>(&mut self, i: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Key>,
        V: Into<Value>,
    {
        i.into_iter().for_each(|(k, v)| {
            self.insert(k.into(), v.into());
        });
    }
}
impl<'s, S> Map<'s, S>
where
    S: StorageWrite,
{
    pub async fn commit(&mut self) -> Result<Addr, Error> {
        let kvs = mem::replace(&mut self.stage, HashMap::new())
            .into_iter()
            .collect::<Vec<_>>();
        if let Some(_) = self.addr.as_ref() {
            unimplemented!("map commit mutate")
        } else {
            prolly::Create::new(self.storage).with_kvs(kvs).await
        }
    }
}
impl<'s, S> Map<'s, S>
where
    S: StorageRead,
{
    pub async fn get<K>(&mut self, k: K) -> Result<Option<Value>, Error>
    where
        K: Into<Key>,
    {
        unimplemented!("map get")
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
