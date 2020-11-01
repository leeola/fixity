use {
    crate::{
        deser::{Deser, Error as DeserError},
        prolly::{
            node::{Node, NodeOwned},
            roller::{Config as RollerConfig, Roller},
        },
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        Addr, Error,
    },
    std::mem,
};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Insert(Value),
    Remove,
}
pub struct Update<'s, S> {
    storage: &'s S,
    root_addr: Addr,
    roller: Roller,
}
impl<'s, S> Update<'s, S> {
    pub fn new(storage: &'s S, root_addr: Addr) -> Self {
        Self::with_roller(storage, root_addr, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, root_addr: Addr, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            root_addr,
            roller: Roller::with_config(roller_config),
        }
    }
}
impl<'s, S> Update<'s, S>
where
    S: StorageWrite + StorageRead,
{
    /// Applies the given changes to the Prolly tree being updated.
    ///
    /// # Errors
    ///
    /// If the provided vec contains non-unique keys or any writes to storage fail
    /// an error is returned.
    pub async fn from_vec(mut self, mut changes: Vec<(Key, Change)>) -> Result<Addr, Error> {
        // Ensure the kvs are sorted - as the trees require sorting.
        // unstable should be fine, since the keys will (soon) be unique.
        changes.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        // Ensure the changes are unique.
        let maybe_dups_len = changes.len();
        changes.dedup_by(|a, b| a.0 == b.0);
        if maybe_dups_len != changes.len() {
            return Err(Error::Prolly {
                message: "cannot construct prolly tree with non-unique keys".to_owned(),
            });
        }
        let removes = changes
            .iter()
            .filter_map(|(k, change)| match change {
                Change::Remove => Some(k.clone()),
                Change::Insert(v) => None,
            })
            .collect::<Vec<_>>();
        let inserts = changes
            .into_iter()
            .filter_map(|(k, change)| match change {
                Change::Remove => None,
                Change::Insert(v) => Some((k, v)),
            })
            .collect::<Vec<_>>();
        todo!()
        // self.recursive_from_kvs(KeyValues::from(kvs)).await
    }
}
