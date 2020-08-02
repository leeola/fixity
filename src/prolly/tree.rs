#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        prolly::RollHasher,
        storage::{Storage, StorageRead, StorageWrite},
        Addr,
    },
    multibase::Base,
    std::collections::HashMap,
};
/// The embed-friendly tree data structure, representing the root of the tree in either
/// values or `Ref<Addr>`s.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Node<K, V> {
    Refs(Vec<(K, Addr)>),
    Values(Vec<(K, V)>),
}
/// The primary constructor implementation to distribute values
pub struct Tree {
}
#[cfg(all(feature = "cjson", feature = "serde"))]
impl<K, V> Node<K, V>
where
    K: Serialize + Clone,
    V: Serialize,
{
    pub fn new<S, I>(storage: &S, sorted_kvs: I) -> Result<Self, String>
    where
        S: StorageWrite,
        I: IntoIterator<Item = (K, V)>,
    {
        let mut roller = RollHasher::new(4);
        let mut headers = Vec::new();
        let mut first_block: Option<Vec<(K, V)>> = None;
        let items = Vec::new();
        for (k, v) in sorted_kvs.into_iter() {
            let boundary =
                roller.roll_bytes(&cjson::to_vec(&k).map_err(|err| format!("{:?}", err))?);
            current_block_items.push((k, v));
            if boundary {
                if first_block.is_some() {
                    let header_key = current_block_items[0].0.clone();
                    // TODO: serialize the block as a Map node or possibly node items.
                    let block_bytes =
                        cjson::to_vec(&current_block_items).map_err(|err| format!("{:?}", err))?;
                    current_block_items.clear();
                    let block_hash = <[u8; 32]>::from(blake3::hash(&block_bytes));
                    let block_addr = multibase::encode(Base::Base58Btc, &block_hash);
                    storage.write(&block_addr, &*block_bytes).unwrap();
                    headers.push((header_key, block_addr));
                } else {
                    let first_block_inner = std::mem::replace(&mut current_block_items, Vec::new());
                    first_block.replace(first_block_inner);
                }
            }
        }
        todo!()
    }
}
