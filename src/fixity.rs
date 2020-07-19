use crate::Result;

pub struct Fixity<S> {
    storage: S,
}
impl<S> Fixity<S> {
    pub fn builder() -> Builder {
        Builder::default()
    }
}
pub struct Builder<S> {
    storage: Option<S>,
}
impl<S> Builder<S> {
    pub fn with_storage(storage: S) -> Self {
        todo!()
    }
    pub fn build() -> Result<Self> {
        todo!()
    }
}
