use {
    crate::{
        primitive::{Build, Flush, GetAddr, InsertAddr, Map},
        value::Key,
        Addr, Error, StorageRead, StorageWrite,
    },
    std::ops::{Deref, DerefMut},
};

pub struct Chain<'s, T> {
    links: Vec<(Key, Box<dyn Link + 's>)>,
    inner: T,
}
impl<'s, T> Chain<'s, T> {
    pub fn build(addr: Option<Addr>) -> Builder<'s> {
        Builder::new(addr)
    }
    pub fn new(inner: T) -> Self {
        Self {
            links: Vec::new(),
            inner,
        }
    }
}
impl<'s, T> std::ops::Deref for Chain<'s, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'s, T> std::ops::DerefMut for Chain<'s, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
#[async_trait::async_trait]
impl<'s, T> Flush for Chain<'s, T>
where
    T: Flush,
{
    async fn flush(&mut self) -> Result<Addr, Error> {
        let mut addr = self.inner.flush().await?;
        let links = &self.links;
        for (key, link) in links.iter().rev() {
            link.insert_addr(key.clone(), addr.clone()).await?;
            addr = link.flush().await?;
        }
        Ok(addr)
    }
}
pub struct Builder<'s> {
    addr: Option<Addr>,
    parents: Vec<(Key, Box<dyn Link + 's>)>,
}
impl<'s> Builder<'s> {
    pub fn new(addr: Option<Addr>) -> Self {
        Self {
            addr,
            parents: Vec::new(),
        }
    }
    pub async fn push<B>(&mut self, key: Key, parent_builder: B) -> Result<(), Error>
    where
        B: Build + 's,
        B::Primitive: Link + GetAddr,
    {
        match self.addr.as_ref() {
            Some(addr) => {
                // NIT: I could take the Addr, and then set it for the next one? Not sure which
                // would be more cheap.
                let addr = Some(addr.clone());
                let parent = parent_builder.build(addr).await?;
                self.addr = parent.get_addr(key.clone()).await?;
                self.parents.push((key, Box::new(parent)));
            }
            None => {
                let parent = parent_builder.build(None).await?;
                self.parents.push((key, Box::new(parent)));
            }
        }
        Ok(())
    }
    pub async fn build<B>(self, builder: B) -> Result<Chain<'s, B::Primitive>, Error>
    where
        B: Build + 's,
        B::Primitive: Flush,
    {
        // NIT: I could take the Addr, and then set it for the next one? Not sure which
        // would be more cheap.
        let addr = self.addr.clone();
        let inner = builder.build(addr).await?;
        Ok(Chain {
            parents: self.parents,
            inner,
        })
    }
}
pub trait Link: Flush + InsertAddr + Sync + Send {}
impl<T> Link for T where T: Flush + InsertAddr + Sync + Send {}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{
            primitive::{Map, MapBuilder},
            storage::Memory,
        },
    };
    #[tokio::test]
    async fn defer() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let mut b = Chain::<Map<'_, Memory>>::build(None);
        b.push("foo".into(), MapBuilder::new(&storage))
            .await
            .unwrap();
        let _d = b.build(MapBuilder::new(&storage)).await.unwrap();
        // let mut m = Map::new(&storage, None);
        // m.append((0..20).map(|i| (i, i * 10)));
        // dbg!(&storage);
    }
}
