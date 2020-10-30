//! A [`crate::prolly`] reference implementation.

use {
    crate::{
        prolly::{
            node::Node,
            roller::{Config as RollerConfig, Roller},
        },
        storage::StorageWrite,
        value::{Key, Value},
        Addr, Error,
    },
    std::mem,
};

/// Create a prolly tree with a cursor, optimized for and requiring sorted insertions.
pub struct Create<'s, S> {
    storage: &'s S,
    roller: Roller,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller: Roller::with_config(roller_config),
        }
    }
}
impl<'s, S> Create<'s, S>
where
    S: StorageWrite,
{
    /// Constructs a prolly tree based on the given `Key, Value` pairs.
    ///
    /// # Errors
    ///
    /// If the provided vec contains non-unique keys or any writes to storage fail
    /// an error is returned.
    pub async fn from_vec(mut self, mut kvs: Vec<(Key, Value)>) -> Result<Addr, Error> {
        // Ensure the kvs are sorted - as the trees require sorting.
        // unstable should be fine, since the keys will (soon) be unique.
        kvs.sort_unstable();
        // Ensure the kvs are unique.
        {
            let maybe_dups_len = kvs.len();
            kvs.dedup_by(|a, b| a.0 == b.0);
            if maybe_dups_len != kvs.len() {
                return Err(Error::Prolly {
                    message: "cannot construct prolly tree with non-unique keys".to_owned(),
                });
            }
        }
        for kv in kvs.into_iter() {
            self.push(kv).await?;
        }
        self.flush().await
    }
    async fn flush(self) -> Result<Addr, Error> {
        todo!()
    }
    async fn push(&self, kv: (Key, Value)) -> Result<Addr, Error> {
        todo!()
    }
}
