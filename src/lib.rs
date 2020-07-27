pub mod error;
pub mod fixity;
pub mod prolly;
mod hash_tree;
pub mod storage;
pub mod store;

#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSerialize};
pub use {
    self::fixity::Fixity,
    error::{Error, Result},
    storage::Storage,
    store::Store,
};

pub struct Id {
    pub rand: String,
    pub signature: String,
}
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub struct Addr(String);
impl From<String> for Addr {
    fn from(hash: String) -> Self {
        Self(hash)
    }
}
impl From<&str> for Addr {
    fn from(hash: &str) -> Self {
        hash.to_owned().into()
    }
}
pub struct Claim {
    pub commit: Commit,
    pub signature: String,
}
pub enum Commit {
    Init { pubkey: String },
    Append { body: CommitBody, prev_commit: Addr },
}
pub enum CommitBody {
    InsertContent {
        id: Id,
        content: Addr,
        prev_content: Option<Addr>,
    },
    DeleteContent {
        id: Id,
    },
}
#[derive(Debug)]
pub enum ContentType {
    Json,
    User(String),
}
#[derive(Debug)]
pub struct ContentHeader {
    pub content_type: ContentType,
    pub metadata: Option<()>,
    pub size: usize,
    pub nodes_count: usize,
    pub chunks_count: usize,
    pub primary_nodes: Vec<Addr>,
    pub root_node: ContentNode,
}
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub struct ContentNode {
    pub children: ContentAddrs,
}
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub enum ContentAddrs {
    Chunks(Vec<Addr>),
    Nodes(Vec<Addr>),
}
impl ContentAddrs {
    pub fn chunks_from<T>(addrs: Vec<T>) -> Self
    where
        T: Into<Addr>,
    {
        Self::Chunks(addrs.into_iter().map(|t| t.into()).collect())
    }
    pub fn nodes_from<T>(addrs: Vec<T>) -> Self
    where
        T: Into<Addr>,
    {
        Self::Nodes(addrs.into_iter().map(|t| t.into()).collect())
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Chunks(v) | Self::Nodes(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Chunks(v) | Self::Nodes(v) => v.is_empty(),
        }
    }
}
