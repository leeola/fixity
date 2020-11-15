use {
    crate::{Addr, Error},
    std::path::Path,
};
pub enum Ref {
    Addr(Addr),
    Ref(String),
}
pub struct Head {
    stage: Addr,
    head: Ref,
}
impl Head {
    pub async fn open(_fixi_dir: &Path, _workspace: &str) -> Result<Self, Error> {
        todo!("open head")
    }
    pub fn addr(&self) -> Option<Addr> {
        todo!("head addr")
    }
}
pub struct Guard<T> {
    head: Head,
    inner: T,
}
impl<T> Guard<T> {
    pub fn new(head: Head, inner: T) -> Self {
        Self { head, inner }
    }
    pub async fn stage(&self) -> Result<Addr, Error> {
        todo!("guard stage")
    }
    pub async fn commit(&self) -> Result<Addr, Error> {
        todo!("guard commit")
    }
}
impl<T> std::ops::Deref for Guard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> std::ops::DerefMut for Guard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
