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
/// A temp error type
type Error = String;
/// The embed-friendly tree data structure, representing the root of the tree in either
/// values or `Ref<Addr>`s.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Node<K, V> {
    Branch(Vec<(K, Addr)>),
    Leaf(Vec<(K, V)>),
}
impl<K, V> Node<K, V> {
    /// Return the first key in this node, if any.
    pub fn first_key(&self) -> Option<&K> {
        match self {
            Self::Branch(v) => v.get(0).map(|(k, _)| k),
            Self::Leaf(v) => v.get(0).map(|(k, _)| k),
        }
    }
    /// Len of the underlying vec.
    pub fn len(&self) -> Option<&K> {
        match self {
            Self::Branch(v) => v.len(),
            Self::Leaf(v) => v.len(),
        }
    }
}
/// The primary constructor implementation to distribute values
pub struct Tree<'s, S, K, V> {
    storage: &'s S,
    depth: usize,
    width: usize,
    root: Layer<'s, S, K, V>,
}
struct Layer<'s, S, K, V> {
    storage: &'s S,
    pattern: u32,
    block: Block<'s, S, K, V>,
}
enum Block<'s, S, K, V> {
    Branch {
        child: Box<Layer<'s, S, K, V>>,
        block: Vec<(K, Addr)>,
    },
    Leaf {
        block: Vec<(K, V)>,
    },
}
impl<'s, S, K, V> Layer<'s, S, K, V> {
    pub fn new(storage: &S) -> Self {
        let mut roller = RollHasher::new(4);
        todo!("layer::new")
    }
}
#[cfg(all(feature = "cjson", feature = "serde"))]
impl<K, V> Layer<'s, S, K, V>
where
    S: StorageWrite,
    K: Serialize + Clone,
    V: Serialize,
{
    pub fn flush(&mut self) -> Result<Option<Node>, Error> {
        todo!("flush")
    }
    pub fn push(&mut self, item: (K, V)) -> Result<Option<Node>, Error> {
        let keys_at_boundary: bool = match &mut self.block {
            Block::Branch { child, block } => {
                // self.handle_child_resp(child.push(item)?)
                // TODO: hash child
                // TODO: write child to storage
                // TODO: rolling hash key
                todo!("child branch")
            }
            Block::Leaf { block } => {
                self.block.push(item);
                // TODO: rolling hash key
                self.roller
                    .roll_bytes(&cjson::to_vec(&k).map_err(|err| format!("{:?}", err))?)
            }
        };
        if keys_at_boundary {
            self.flush()
        } else {
            Ok(None)
        }
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::storage::{Memory, Storage, StorageRead, StorageWrite},
        maplit::hashmap,
    };
    #[test]
    fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
    }
}
