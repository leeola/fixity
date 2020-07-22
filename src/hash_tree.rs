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
    pub fn push<A: Into<Addr>>(self, hash: A) -> (Self, Option<ContentNode>) {
        let hash = Addr::from(hash);
        let (expand, content_node) = self.root_node.push(hash);
        if expand {
            let Self {
                mut depth,
                root_node,
            } = self;
            depth += 1;
            let root_node = Node::new(depth, max_hashes, root_node);
            (Self { depth, root_node }, content_node)
        } else {
            (self, content_node)
        }
    }
}
#[derive(Debug)]
struct Node {
    depth: usize,
    max_hashes: usize,
    hashes: Vec<Addr>,
    child: Option<Box<Node>>,
    state: NodeState,
}
impl Node {
    pub fn new(depth: usize, max_hashes: usize, child: Option<Node>) -> Self {
        Self {
            depth,
            max_hashes,
            child: child.map(Box::new),
            hashes: Vec::new(),
            state: NodeState::ReceiveHash,
        }
    }
    pub fn push(&mut self, hash: Addr) -> (bool, Option<ContentNode>) {
        match (self.depth, self.state) {
            (0, NodeState::ReceiveHash) => {
                self.hashes.push(hash);
                if self.hashes.len() == self.max_hashes {
                    let hashes = mem::replace(&mut self.hashes, Vec::new());
                    return (
                        true,
                        Some(ContentNode {
                            children: ContentAddrs::Chunks(hashes),
                        }),
                    );
                }
                return (false, None);
            }
            (_, NodeState::ReceiveHash) => {
                self.state = NodeState::ProxyHash;
                self.hashes.push(hash);
                if self.hashes.len() == self.max_hashes {
                    let hashes = mem::replace(&mut self.hashes, Vec::new());
                    return (
                        false,
                        Some(ContentNode {
                            children: ContentAddrs::Nodes(hashes),
                        }),
                    );
                }
                return (false, None);
            }
            (0, NodeState::ProxyHash) => unreachable!(),
            (_, NodeState::ProxyHash) => {
                let child = self.child.as_mut().expect("proxy missing child");
                let (_, hashes) = child.push(hash);
                if hashes.is_some() {
                    self.state = NodeState::ReceiveHash;
                }
                return (false, None);
            }
        }
    }
}
#[derive(Debug, Copy, Clone)]
enum NodeState {
    ReceiveHash,
    ProxyHash,
}
