pub type Result<T> = std::result::Result<T, ()>;
pub trait Fixity {
    fn new() -> Ident;
    fn push<T>(content: T, id: Option<Ident>) -> Result<Commit>;
    fn clone() -> ();
}
pub struct Blob {
    pub blob: String,
}
pub struct Ident {
    pub rand: String,
    pub signature: String,
}
pub struct Addr(String);
pub struct Claim {
    pub commit: Commit,
    pub signature: String,
}
pub enum Commit {
    Init,
    Append { body: CommitBody, prev_commit: Addr },
}
pub enum CommitBody {
    InsertContent {
        ident: Ident,
        content: Addr,
        prev_content: Option<Addr>,
    },
    DeleteContent {
        ident: Ident,
    },
}
pub struct BytesHeader {
    pub bytes_count: usize,
    pub parts_count: usize,
    pub chunks_count: usize,
    pub first_part: Addr,
}
pub struct BytesPart {
    pub part_bytes_count: usize,
    pub part_chunks_count: u16,
    pub chunks: Vec<Addr>,
    pub next_part: Option<Addr>,
}
