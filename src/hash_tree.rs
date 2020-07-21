use {
    crate::{Addr, ContentAddrs, ContentNode, Fixity, Result},
    fastcdc::Chunk,
    std::mem,
};
#[derive(Debug)]
pub struct HashTree {
    depth: usize,
    root_node: Node,
}
impl HashTree {
    pub fn new(max_hashes: usize) -> Self {
        Self {
            depth: 0,
            root_node: Node::new(max_hashes, None),
        }
    }
    pub fn push<A: Into<Addr>>(hash: A) -> Option<ContentNode> {
        // let hash = Addr::from(hash);
        // self.root_node.push(hash);
        todo!()
    }
}
#[derive(Debug)]
pub struct Node {
    depth: usize,
    max_hashes: usize,
    hashes: Vec<Addr>,
    child: Option<Box<Node>>,
    state: NodeState,
}
impl Node {
    pub fn new(max_hashes: usize, child: Option<Node>) -> Self {
        Self {
            depth: 0,
            max_hashes,
            child: child.map(Box::new),
            hashes: Vec::new(),
            state: NodeState::ReceiveHash,
        }
    }
    pub fn into_parent(self) -> Self {
        todo!()
    }
    pub fn push(&mut self, hash: Addr) -> Option<ContentNode> {
        match (self.depth, self.state) {
            (0, NodeState::ReceiveHash) => {
                self.hashes.push(hash);
                if self.hashes.len() == self.max_hashes {
                    let hashes = mem::replace(&mut self.hashes, Vec::new());
                    return Some(ContentNode {
                        children: ContentAddrs::Chunks(hashes),
                    });
                }
                return None;
            }
            (_, NodeState::ReceiveHash) => {
                self.state = NodeState::ProxyHash;
                self.hashes.push(hash);
                if self.hashes.len() == self.max_hashes {
                    let hashes = mem::replace(&mut self.hashes, Vec::new());
                    return Some(ContentNode {
                        children: ContentAddrs::Nodes(hashes),
                    });
                }
                return None;
            }
            (0, NodeState::ProxyHash) => unreachable!(),
            (_, NodeState::ProxyHash) => {
                let child = self.child.as_mut().expect("proxy missing child");
                let hashes = child.push(hash);
                if hashes.is_some() {
                    self.state = NodeState::ReceiveHash;
                }
            }
        }
        todo!()
    }
}
#[derive(Debug)]
enum NodeState {
    ReceiveHash,
    ProxyHash,
}
