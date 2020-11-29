use {
    crate::{fixity::Flush, primitive::Map, value::Key, Addr, Error, StorageRead, StorageWrite},
    std::ops::{Deref, DerefMut},
};

#[async_trait::async_trait]
pub trait DeferTo: Insert + Flush {}
#[async_trait::async_trait]
pub trait Init: Sized {
    async fn defer_init(addr: Option<Addr>) -> Result<Self, Error>;
}
#[async_trait::async_trait]
pub trait BuildPrimitive {
    type Primitive: DeferTo + Get;
    async fn build(self, addr: Option<Addr>) -> Result<Self::Primitive, Error>;
}
#[async_trait::async_trait]
pub trait Insert {
    async fn defer_insert(&mut self, key: Key, addr: Addr) -> Result<(), Error>;
}
#[async_trait::async_trait]
pub trait Get {
    async fn defer_get(&self, key: Key) -> Result<Addr, Error>;
}

pub struct Defer<'s, T> {
    parents: Vec<(Key, Box<dyn DeferTo + 's>)>,
    inner: T,
}
impl<'s, T> Defer<'s, T> {
    pub fn build(addr: Option<Addr>) -> Builder<'s> {
        Builder::new(addr)
    }
    // pub fn new(inner: T) -> Self {
    //     Self {
    //         parents: Vec::new(),
    //         inner,
    //     }
    // }
    // pub fn push<To>(&mut self, key: Key, to: To)
    // where
    //     To: DeferTo + 'static,
    // {
    //     self.parents.push(Box::new(to));
    // }
}
impl<'s, T> std::ops::Deref for Defer<'s, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'s, T> std::ops::DerefMut for Defer<'s, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
pub struct Builder<'s> {
    addr: Option<Addr>,
    parents: Vec<(Key, Box<dyn DeferTo + 's>)>,
}
impl<'s> Builder<'s> {
    pub fn new(addr: Option<Addr>) -> Self {
        Self {
            addr,
            parents: Vec::new(),
        }
    }
    pub async fn push<ParentBuilder>(
        &mut self,
        key: Key,
        parent_builder: ParentBuilder,
    ) -> Result<(), Error>
    where
        ParentBuilder: BuildPrimitive + 's,
    {
        match self.addr.as_ref() {
            Some(addr) => {
                // NIT: I could take the Addr, and then set it for the next one? Not sure which
                // would be more cheap.
                let addr = Some(addr.clone());
                let parent = parent_builder.build(addr).await?;
                self.addr.replace(parent.defer_get(key.clone()).await?);
                self.parents.push((key, Box::new(parent)));
            }
            None => {
                let parent = parent_builder.build(None).await?;
                self.parents.push((key, Box::new(parent)));
            }
        }
        Ok(())
    }
    pub async fn build<B>(
        self,
        key: Key,
        primitive_builder: B,
    ) -> Result<Defer<'s, B::Primitive>, Error>
    where
        B: BuildPrimitive + 's,
    {
        // NIT: I could take the Addr, and then set it for the next one? Not sure which
        // would be more cheap.
        let addr = self.addr.clone();
        let inner = primitive_builder.build(addr).await?;
        Ok(Defer {
            parents: self.parents,
            inner,
        })
    }
}
// #[async_trait::async_trait]
// impl<'s, S> Init for Map<'s, S>
// where
//     S: StorageRead,
// {
//     async fn defer_init(addr: Option<Addr>) -> Result<Self, Error> {
//         todo!("defer init")
//     }
// }
pub struct MapBuilder<'s, S> {
    storage: &'s S,
}
#[async_trait::async_trait]
impl<'s, S> BuildPrimitive for MapBuilder<'s, S>
where
    S: StorageRead + StorageWrite,
{
    type Primitive = Map<'s, S>;
    async fn build(self, addr: Option<Addr>) -> Result<Self::Primitive, Error> {
        Ok(Map::new(self.storage, addr))
    }
}
#[async_trait::async_trait]
impl<'s, S> DeferTo for Map<'s, S> where S: StorageRead + StorageWrite {}
#[async_trait::async_trait]
impl<'s, S> Insert for Map<'s, S>
where
    S: StorageWrite,
{
    async fn defer_insert(&mut self, key: Key, addr: Addr) -> Result<(), Error> {
        todo!("defer insert")
    }
}
#[async_trait::async_trait]
impl<'s, S> Get for Map<'s, S>
where
    S: StorageRead,
{
    async fn defer_get(&self, key: Key) -> Result<Addr, Error> {
        todo!("defer get")
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{primitive::Map, storage::Memory},
    };
    #[test]
    fn defer() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let mut b = Defer::<Map<'_, Memory>>::build(None);
        b.push("foo".into(), MapBuilder { storage: &storage });
        let _d = b.build("foo".into(), MapBuilder { storage: &storage });
        // let mut m = Map::new(&storage, None);
        // m.append((0..20).map(|i| (i, i * 10)));
        // dbg!(&storage);
    }
}
