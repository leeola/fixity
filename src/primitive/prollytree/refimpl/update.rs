use {
    crate::{
        primitive::prollytree::{
            refimpl::{Create, Read},
            roller::Config as RollerConfig,
        },
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        Addr, Error,
    },
    std::collections::HashMap,
};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Insert(Value),
    Remove,
}
pub struct Update<'s, S> {
    storage: &'s S,
    root_addr: Addr,
    roller_config: RollerConfig,
}
impl<'s, S> Update<'s, S> {
    pub fn new(storage: &'s S, root_addr: Addr) -> Self {
        Self::with_roller(storage, root_addr, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, root_addr: Addr, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            root_addr,
            roller_config,
        }
    }
}
impl<'s, S> Update<'s, S>
where
    S: StorageWrite + StorageRead,
{
    /// Applies the given changes to the Prolly tree being updated.
    ///
    /// For safety a key can only be modified once in the given changes vec. This ensures
    /// multiple changes are not applied to the source tree in an unexpected order.
    ///
    /// # Errors
    ///
    /// If the provided vec contains non-unique keys or any writes to storage fail
    /// an error is returned.
    pub async fn with_vec(self, mut changes: Vec<(Key, Change)>) -> Result<Addr, Error> {
        {
            changes.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            // Ensure the changes are unique.
            let maybe_dups_len = changes.len();
            changes.dedup_by(|a, b| a.0 == b.0);
            if maybe_dups_len != changes.len() {
                return Err(Error::Prolly {
                    message: "cannot construct prolly tree with non-unique keys".to_owned(),
                });
            }
        }
        let all_kvs = Read::new(self.storage, self.root_addr.clone())
            .to_vec()
            .await?;
        let mut kvs = all_kvs
            .into_iter()
            // Filter out any Keys that were removed.
            .filter(|(source_key, _)| {
                let change = changes
                    .iter()
                    .find(|(changed_key, _)| source_key == changed_key)
                    .map(|(_, change)| change);
                let key_is_removed = matches!(change, Some(Change::Remove));
                // if the key is removed, return false to drop it from the vec via filter.
                !key_is_removed
            })
            // Collecting into a hashmap allows us to uniquely apply insertions.
            .collect::<HashMap<_, _>>();
        // insert any changes that were insertions.
        changes
            .into_iter()
            // ignore non-insert changes.
            .filter_map(|(changed_key, change)| match change {
                Change::Remove => None,
                Change::Insert(changed_value) => Some((changed_key, changed_value)),
            })
            .for_each(|(k, v)| {
                kvs.insert(k, v);
            });
        // kvs is now the modified vec of keyvalues and can be constructed as an entire
        // new tree. Since each block is deterministic, this effectively mutates the source
        // tree with the least lines of code.
        Create::with_roller(self.storage, self.roller_config)
            .with_hashmap(kvs)
            .await
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::storage::Memory};
    /// A smaller value to use with the roller, producing smaller average block sizes.
    const TEST_PATTERN: u32 = (1 << 8) - 1;
    #[tokio::test]
    async fn apply_compare_updates() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();

        use TestChange::*;
        let test_cases = vec![
            ((0..3), vec![(0, Remove)], vec![(1..3)]),
            (
                (0..7),
                vec![(0, Remove), (7, Insert), (8, Insert), (9, Insert)],
                vec![(1..10)],
            ),
        ];

        for (source_kvs, changes, expected_kvs) in test_cases.into_iter() {
            let expected_kvs = expected_kvs
                .into_iter()
                .flatten()
                .map(|i| (Key::from(i), Value::from(i)))
                .collect::<Vec<_>>();
            let storage = Memory::new();
            let source_addr = {
                let tree = Create::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
                let source_kvs = source_kvs
                    .map(|i| (i, i))
                    .map(|(k, v)| (Key::from(k), Value::from(v)))
                    .collect::<Vec<_>>();
                tree.with_vec(source_kvs).await.unwrap()
            };
            let got_addr = {
                Update::with_roller(
                    &storage,
                    source_addr,
                    RollerConfig::with_pattern(TEST_PATTERN),
                )
                .with_vec(
                    changes
                        .into_iter()
                        .map(|(i, change)| match change {
                            Remove => (Key::from(i), Change::Remove),
                            Insert => (Key::from(i), Change::Insert(Value::from(i))),
                        })
                        .collect::<Vec<_>>(),
                )
                .await
                .unwrap()
            };
            let got_kvs = Read::new(&storage, got_addr.clone())
                .to_vec()
                .await
                .unwrap();
            assert_eq!(expected_kvs, got_kvs);
            let expected_addr = {
                Create::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN))
                    .with_vec(expected_kvs)
                    .await
                    .unwrap()
            };
            assert_eq!(expected_addr, got_addr,);
        }
    }
    enum TestChange {
        Remove,
        Insert,
    }
}
