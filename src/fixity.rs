use {
    crate::{storage::Storage, Error, Result, Store},
    std::io::{Read, Write},
};
pub struct Fixity<S> {
    storage: S,
}
impl<S> Fixity<S> {
    pub fn builder() -> Builder<S> {
        Builder::new()
    }
}
impl<S> Store for Fixity<S>
where
    S: Storage,
{
    fn put(&self, bytes: &mut dyn Read, hashes: &mut dyn Write) -> Result<usize> {
        todo!()
    }
}
pub struct Builder<S> {
    storage: Option<S>,
}
impl<S> Builder<S> {
    pub fn new() -> Self {
        Self { storage: None }
    }
    pub fn with_storage(mut self, storage: S) -> Self {
        self.storage.replace(storage);
        self
    }
    pub fn build(self) -> Result<Fixity<S>> {
        let storage = self.storage.ok_or_else(|| Error::BuilderError {
            message: "must call Builder::with_storage to build".into(),
        })?;
        Ok(Fixity { storage })
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::storage::{Memory, StorageRead, StorageWrite},
    };
    #[test]
    fn build() {
        let fixi = Fixity::builder()
            .with_storage(Memory::default())
            .build()
            .unwrap();
    }
}
