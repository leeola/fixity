pub mod error;
pub mod fixity;
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
#[derive(Debug)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub struct Addr(String);
impl From<String> for Addr {
    fn from(hash: String) -> Self {
        Self(hash)
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
pub struct BytesHeader {
    pub content_type: ContentType,
    pub metadata: Option<()>,
    pub bytes_count: usize,
    pub parts_count: usize,
    pub blobs_count: usize,
    pub primary_nodes: Vec<ContentAddr>,
    pub root_node: ContentThing,
}
#[derive(Debug)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub enum ContentAddr {
    Bytes(Addr),
    Node(Addr),
}
#[derive(Debug)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub enum BytesAddrs {
    Blobs(Vec<Addr>),
    Parts(Vec<Addr>),
}
impl BytesAddrs {
    pub fn len(&self) -> usize {
        match self {
            Self::Blobs(v) | Self::Parts(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Blobs(v) | Self::Parts(v) => v.is_empty(),
        }
    }
}
pub enum BytesNode {
    Blobs(BytesBlobs),
    Part(BytesPart),
}
#[derive(Debug, Default)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub struct BytesBlobs {
    pub bytes_count: u64,
    pub blobs: Vec<Addr>,
}
#[derive(Debug)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
pub struct BytesPart {
    pub bytes_count: u64,
    pub addrs: BytesAddrs,
}
