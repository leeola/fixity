use crate::Result;
pub trait Fixity {
    fn new() -> Id;
    fn push<T>(content: T, id: Option<Id>) -> Result<Commit>;
    fn clone() -> ();
}
pub struct Id {
    pub rand: String,
    pub signature: String,
}
pub struct Addr(String);
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
pub struct BytesHeader {
    pub content_type: ContentType,
    pub metadata: Option<()>,
    pub bytes_count: usize,
    pub parts_count: usize,
    pub blobs_count: usize,
    pub first_part: Addr,
}
pub struct BytesPart {
    pub part_bytes_count: usize,
    pub part_chunks_count: u16,
    pub blobs: Vec<Addr>,
    pub next_part: Option<Addr>,
}
