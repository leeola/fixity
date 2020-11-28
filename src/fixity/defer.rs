use {
    crate::{fixity::Flush, value::Key, Addr, Error},
    std::ops::{Deref, DerefMut},
};

pub trait DeferTo: Insert + Flush {}
pub trait Init: Sized {
    fn defer_init(addr: Option<Addr>) -> Result<Self, Error>;
}
pub trait Insert {
    fn defer_insert(&self, key: Key, addr: Addr) -> Result<Addr, Error>;
}

pub struct Defer<T> {
    parents: Vec<(Key, Box<dyn DeferTo>)>,
    inner: T,
}
impl<T> Defer<T> {
    pub fn build(addr: Option<Addr>) -> Builder {
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
impl<T> std::ops::Deref for Defer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> std::ops::DerefMut for Defer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
pub struct Builder {
    addr: Option<Addr>,
    parents: Vec<(Key, Box<dyn DeferTo>)>,
}
impl Builder {
    pub fn new(addr: Option<Addr>) -> Self {
        Self {
            addr,
            parents: Vec::new(),
        }
    }
    pub fn push<Parent>(&mut self, key: Key)
    where
        Parent: DeferTo + 'static,
    {
        // self.parents.push(Box::new(to));
        todo!("push")
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{primitive::Map, storage::Memory},
    };
    #[test]
    fn dyn_defer() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let _d = DynDefer::<Map<'_, Memory>, Map<'_, Memory>>::new();
        // let mut m = Map::new(&storage, None);
        // m.append((0..20).map(|i| (i, i * 10)));
        // dbg!(&storage);
    }
    #[test]
    fn defer() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        // let _d = Defer::<Map>::new(None);
        // let mut m = Map::new(&storage, None);
        // m.append((0..20).map(|i| (i, i * 10)));
        // dbg!(&storage);
    }
}
