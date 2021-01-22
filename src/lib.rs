#![feature(generic_associated_types)]

pub mod deser;
mod dir;
pub mod error;
pub mod fixity;
pub mod head;
pub mod map;
pub mod path;
pub mod primitive;
pub mod prolly;
pub mod storage;
pub mod value;
pub mod workspace;

pub use self::{
    error::{Error, Result},
    fixity::Fixity,
    map::Map,
    storage::{Storage, StorageRead, StorageWrite},
    value::{Addr, Path},
};

/*
pub struct Id {
    pub rand: String,
    pub signature: String,
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
*/
