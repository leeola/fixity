pub mod error;
pub mod fixity;
pub mod storage;
pub mod store;

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
pub enum ContentType {
    Json,
    User(String),
}
#[derive(Debug, Default)]
pub struct BytesHeader {
    pub content_type: ContentType,
    pub metadata: Option<()>,
    pub bytes_count: usize,
    pub parts_count: usize,
    pub blobs_count: usize,
    // pub first_part: BytesLayer,
}
pub enum BytesLayer {
    Blobs(Vec<Addr>),
    Parts(Vec<Addr>),
}
#[derive(Debug, Default)]
pub struct BytesPart {
    pub bytes_count: usize,
    pub blobs: Vec<Addr>,
}
#[derive(Debug, Default)]
pub struct BytesLayerPart {
    pub bytes_count: usize,
    pub parts: Vec<Addr>,
}
