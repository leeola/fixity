use {
    crate::{Addr, ContentAddrs, ContentNode, Fixity, Result},
    fastcdc::Chunk,
    std::mem,
};
#[derive(Debug)]
pub struct HashTree {
    depth: usize,
    width: usize,
    root_node: Node,
}
impl HashTree {
    pub fn new(width: usize) -> Self {
        let depth = 0;
        Self {
            depth,
            width,
            root_node: Node::new(depth, width, None),
        }
    }
    pub fn push<A: Into<Addr>>(mut self, hash: A) -> (Self, Option<ContentNode>) {
        let (child_depth, content_node) = self.root_node.push(hash.into());
        if content_node.is_some() && child_depth == self.depth {
            let Self {
                mut depth,
                width,
                root_node,
            } = self;
            depth += 1;
            let root_node = Node::new(depth, width, Some(root_node));
            (
                Self {
                    depth,
                    width,
                    root_node,
                },
                content_node,
            )
        } else {
            (self, content_node)
        }
    }
    #[cfg(test)]
    fn calculate_depth(&self) -> (usize, usize) {
        (self.depth, self.root_node.calculate_depth())
    }
}
#[derive(Debug)]
struct Node {
    depth: usize,
    width: usize,
    hashes: Vec<Addr>,
    child: Option<Box<Node>>,
    state: NodeState,
}
impl Node {
    pub fn new(depth: usize, width: usize, child: Option<Node>) -> Self {
        Self {
            depth,
            width,
            child: child.map(Box::new),
            hashes: Vec::new(),
            state: NodeState::ReceiveHash,
        }
    }
    #[cfg(test)]
    fn calculate_depth(&self) -> usize {
        self.child
            .as_ref()
            .map(|child| child.calculate_depth() + 1)
            .unwrap_or_default()
    }
    pub fn push(&mut self, hash: Addr) -> (usize, Option<ContentNode>) {
        match (self.state, self.child.as_mut()) {
            (NodeState::ReceiveHash, None) => {
                self.hashes.push(hash);
                if self.hashes.len() == self.width {
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
                if self.hashes.len() == self.width {
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
                if content_node.is_some() && depth == self.depth - 1 {
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
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::storage::{Memory, StorageRead, StorageWrite},
    };
    macro_rules! assert_push {
        ($push_ret:expr, $expect_addrs:expr) => {{
            let (tree, node) = $push_ret;
            assert_eq!(node, $expect_addrs.map(|children| ContentNode { children }));
            tree
        }};
    }
    fn chunks<T>(addrs: Vec<T>) -> Option<ContentAddrs>
    where
        T: Into<Addr>,
    {
        Some(ContentAddrs::chunks_from(addrs))
    }
    fn nodes<T>(addrs: Vec<T>) -> Option<ContentAddrs>
    where
        T: Into<Addr>,
    {
        Some(ContentAddrs::nodes_from(addrs))
    }
    #[test]
    fn small_writes() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();

        let tree = HashTree::new(2);
        assert_eq!(tree.calculate_depth(), (0, 0));
        let tree = assert_push!(tree.push("a"), None);
        let tree = assert_push!(tree.push("b"), chunks(vec!["a", "b"]));
        let tree = assert_push!(tree.push("ab"), None);

        let tree = assert_push!(tree.push("c"), None);
        let tree = assert_push!(tree.push("d"), chunks(vec!["c", "d"]));
        let tree = assert_push!(tree.push("cd"), nodes(vec!["ab", "cd"]));
        let tree = assert_push!(tree.push("depth 2 hash 1"), None);

        let tree = assert_push!(tree.push("a"), None);
        let tree = assert_push!(tree.push("b"), chunks(vec!["a", "b"]));
        let tree = assert_push!(tree.push("ab"), None);
        let tree = assert_push!(tree.push("c"), None);
        let tree = assert_push!(tree.push("d"), chunks(vec!["c", "d"]));
        let tree = assert_push!(tree.push("cd"), nodes(vec!["ab", "cd"]));
        let tree = assert_push!(
            tree.push("depth 2 hash 2"),
            nodes(vec!["depth 2 hash 1", "depth 2 hash 2"])
        );
        let tree = assert_push!(tree.push("depth 3 hash 1"), None);

        let tree = assert_push!(tree.push("a"), None);
        let tree = assert_push!(tree.push("b"), chunks(vec!["a", "b"]));
        let tree = assert_push!(tree.push("ab"), None);
        let tree = assert_push!(tree.push("c"), None);
        let tree = assert_push!(tree.push("d"), chunks(vec!["c", "d"]));
        let tree = assert_push!(tree.push("cd"), nodes(vec!["ab", "cd"]));
        let tree = assert_push!(tree.push("depth 2 hash 1"), None);
        let tree = assert_push!(tree.push("a"), None);
        let tree = assert_push!(tree.push("b"), chunks(vec!["a", "b"]));
        let tree = assert_push!(tree.push("ab"), None);
        let tree = assert_push!(tree.push("c"), None);
        let tree = assert_push!(tree.push("d"), chunks(vec!["c", "d"]));
        let tree = assert_push!(tree.push("cd"), nodes(vec!["ab", "cd"]));
        let tree = assert_push!(
            tree.push("depth 2 hash 2"),
            nodes(vec!["depth 2 hash 1", "depth 2 hash 2"])
        );
        let tree = assert_push!(
            tree.push("depth 3 hash 2"),
            nodes(vec!["depth 3 hash 1", "depth 3 hash 2"])
        );
    }
}
