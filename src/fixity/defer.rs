use {
    crate::{fixity::Flush, value::Key, Addr},
    std::ops::{Deref, DerefMut},
};

pub trait DeferTo: DeferredInsert + Flush {}
pub trait Init {}
pub trait DeferredInsert {}

pub struct Defer<T> {
    parents: Vec<(Key, Box<dyn DeferTo>)>,
    inner: T,
}
impl<T> Defer<T> {
    pub fn new(inner: T) -> Self {
        Self {
            parents: Vec::new(),
            inner,
        }
    }
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
    root_addr: Addr,
    parents: Vec<(Key, Box<dyn DeferTo>)>,
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
