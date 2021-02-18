use crate::{
    deser::Deser,
    primitive::prollylist::{Node, NodeOwned},
    storage::StorageRead,
    value::{Addr, Value},
    Error,
};
pub struct Read<'s, S> {
    storage: &'s S,
    root_addr: Addr,
}
impl<'s, S> Read<'s, S> {
    /// Construct a new Read.
    pub fn new(storage: &'s S, root_addr: Addr) -> Self {
        Self { storage, root_addr }
    }
}
impl<'s, S> Read<'s, S>
where
    S: StorageRead,
{
    pub async fn to_vec(&self) -> Result<Vec<Value>, Error> {
        self.recursive_to_vec(self.root_addr.clone()).await
    }
    #[async_recursion::async_recursion]
    async fn recursive_to_vec(&self, addr: Addr) -> Result<Vec<Value>, Error> {
        let mut buf = Vec::new();
        self.storage.read(addr.clone(), &mut buf).await?;
        let node = Deser::default().from_slice::<NodeOwned>(&buf)?;
        match node {
            Node::Leaf(v) => Ok(v),
            Node::Branch(v) => {
                let mut values = Vec::new();
                for addr in v {
                    values.append(&mut self.recursive_to_vec(addr).await?);
                }
                Ok(values)
            },
        }
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{
            primitive::{prollylist::refimpl::Create, prollytree::roller::Config as RollerConfig},
            storage::Memory,
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
        let contents = vec![(0u32..20), (0..200), (0..2_000)];
        for content in contents {
            let written_values = content.map(|v| Value::from(v)).collect::<Vec<_>>();
            let storage = Memory::new();
            let tree = Create::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
            let addr = tree.with_vec(written_values.clone()).await.unwrap();
            let read_values = Read::new(&storage, addr).to_vec().await.unwrap();
            assert_eq!(
                written_values, read_values,
                "expected read values to match written values"
            );
        }
    }
}
