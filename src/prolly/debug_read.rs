use {
    crate::{
        prolly::node::{Node, NodeOwned},
        storage::StorageRead,
        value::{Addr, Key, Value},
        Error,
    },
    std::collections::HashMap,
};

#[derive(Debug)]
pub enum DebugNode {
    Leaf {
        depth: usize,
        kvs: Vec<(Key, Value)>,
    },
    Branch {
        depth: usize,
        kvs: Vec<DebugNodeBranch>,
    },
}
#[derive(Debug)]
pub struct DebugNodeBranch {
    key: Key,
    addr: Addr,
    child: DebugNode,
}
impl DebugNode {
    pub fn print(&self) {
        match self {
            Self::Leaf { depth, kvs } => {
                log::info!("{}Leaf", "  ".repeat(*depth));
                for kv in kvs {
                    log::info!("{}-- {}; {}", "  ".repeat(*depth), kv.0, kv.1);
                }
            }
            Self::Branch { depth, kvs } => {
                log::info!("{}Branch", "  ".repeat(*depth));
                for kv in kvs {
                    log::info!("{}-- {}; {}", "  ".repeat(*depth), kv.key, kv.addr);
                    kv.child.print();
                }
            }
        }
    }
    pub async fn new<S>(storage: &S, addr: &Addr) -> Result<Self, Error>
    where
        S: StorageRead + Sync,
    {
        Self::from_depth_addr(storage, 0, addr).await
    }
    #[async_recursion::async_recursion]
    async fn from_depth_addr<S>(storage: &S, depth: usize, addr: &Addr) -> Result<Self, Error>
    where
        S: StorageRead + Sync,
    {
        let node = {
            let mut buf = Vec::new();
            storage.read(addr.clone(), &mut buf).await?;
            crate::value::deserialize_with_addr::<NodeOwned>(&buf, &addr)?
        };
        match node {
            Node::Leaf(v) => Ok(Self::Leaf { depth, kvs: v }),
            Node::Branch(v) => {
                let mut kvs = Vec::new();
                for (key, addr) in v {
                    let child = Self::from_depth_addr(storage, depth + 1, &addr).await?;
                    kvs.push(DebugNodeBranch { key, addr, child });
                }
                Ok(Self::Branch { depth, kvs })
            }
        }
    }
}
