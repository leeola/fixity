use {
    crate::{
        core::{
            cache::{CacheRead, OwnedRef},
            primitive::prollytree::Node,
        },
        Addr, Error, Key, Value,
    },
    std::convert::TryInto,
};
pub struct Read<'s, C> {
    storage: &'s C,
    root_addr: Addr,
}
impl<'s, C> Read<'s, C> {
    /// Construct a new Read.
    pub fn new(storage: &'s C, root_addr: Addr) -> Self {
        Self { storage, root_addr }
    }
}
impl<'s, C> Read<'s, C>
where
    C: CacheRead,
{
    pub async fn to_vec(&self) -> Result<Vec<(Key, Value)>, Error> {
        self.recursive_to_vec(self.root_addr.clone()).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_to_vec(&self, addr: Addr) -> Result<Vec<(Key, Value)>, Error> {
        let owned_ref = self.storage.read_structured(&addr).await?;
        let node = owned_ref.into_owned_structured().try_into()?;
        match node {
            Node::Leaf(v) => Ok(v),
            Node::Branch(v) => {
                let mut kvs = Vec::new();
                for (_, addr) in v {
                    kvs.append(&mut self.recursive_to_vec(addr).await?);
                }
                Ok(kvs)
            },
        }
    }
    pub async fn get(&self, k: &Key) -> Result<Option<Value>, Error> {
        // TODO: This is perhaps ignoring performance too excessively - even for a refimpl -
        // might want to tweak this. It could be the same LOC as `to_vec` and `recursive_to_vec`,
        // me thinks.
        let v = self.to_vec().await?;
        Ok(v.into_iter().find(|(rhs, _)| k == rhs).map(|(_, v)| v))
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::core::{
            primitive::prollytree::{refimpl::Create, roller::Config as RollerConfig},
            Fixity,
        },
    };
    /// A smaller value to use with the roller, producing smaller average block sizes.
    const TEST_PATTERN: u32 = (1 << 8) - 1;
    #[tokio::test]
    async fn read() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let contents = vec![(0..20), (0..200), (0..2_000)];
        for content in contents {
            let written_kvs = content
                .map(|i| (i, i * 10))
                .map(|(k, v)| (Key::from(k), Value::from(v)))
                .collect::<Vec<_>>();
            let cache = Fixity::memory().into_cache();
            let tree = Create::with_roller(&cache, RollerConfig::with_pattern(TEST_PATTERN));
            let addr = tree.with_vec(written_kvs.clone()).await.unwrap();
            let read_kvs = Read::new(&cache, addr).to_vec().await.unwrap();
            assert_eq!(
                written_kvs, read_kvs,
                "expected read kvs to match written kvs"
            );
        }
    }
}
