use {
    crate::{Addr, ContentAddrs, ContentNode, Fixity, Result},
    fastcdc::Chunk,
    std::mem,
};
#[derive(Debug)]
pub struct HashTree {
    depth: usize,
    max_hashes,
    root_node: Node,
}
impl HashTree {
    pub fn new(max_hashes: usize) -> Self {
        let depth = 0;
        Self {
            depth,
            max_hashes,
            root_node: Node::new(depth, max_hashes, None),
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
            let root_node = Node::new(depth, max_hashes, Some(root_node));
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
    pub fn push(&mut self, hash: Addr) -> (usize, Option<ContentNode>) {
        match (self.state, self.child.as_mut()) {
            (NodeState::ReceiveHash, None) => {
                self.hashes.push(hash);
                if self.hashes.len() == self.max_hashes {
                    let hashes = mem::replace(&mut self.hashes, Vec::new());
                    return (
                        self.depth,
                        Some(ContentNode {
                            children: ContentAddrs::Chunks(hashes),
                        }),
                    );
                }
                return (self.depth, None);
            }
            (NodeState::ReceiveHash, Some(_)) => {
                self.state = NodeState::ProxyHash;
                self.hashes.push(hash);
                if self.hashes.len() == self.max_hashes {
                    let hashes = mem::replace(&mut self.hashes, Vec::new());
                    return (
                        self.depth,
                        Some(ContentNode {
                            children: ContentAddrs::Nodes(hashes),
                        }),
                    );
                }
                return (self.depth, None);
            }
            (NodeState::ProxyHash, None) => unreachable!(),
            (NodeState::ProxyHash, Some(child)) => {
                let (depth, content_node) = child.push(hash);
                // if the child directly below this node finished a node, this
                // node needs to receive the next hash.
                if content_node.is_some() &&
                depth == self.depth-1 {
                    self.state = NodeState::ReceiveHash;
                }
                return (depth, content_node);
            }
        }
    }
}
#[derive(Debug, Copy, Clone)]
enum NodeState {
    ReceiveHash,
    ProxyHash,
}
