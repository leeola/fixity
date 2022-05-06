pub mod memory;
pub use memory::Memory;
use {
    async_trait::async_trait,
    std::{ops::Deref, str, sync::Arc},
};
type Error = ();
#[async_trait]
pub trait ContentStorage<Cid>: Send + Sync
where
    Cid: Send + Sync,
{
    type Content: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn exists(&self, cid: &Cid) -> Result<bool, Error>;
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Content, Error>;
    // TODO: Make this take a Into<Vec<u8>> + AsRef<[u8]>. Not gaining anything by requiring
    // the extra From<Vec<u8>> bound.
    async fn write_unchecked<Content>(&self, cid: Cid, content: Content) -> Result<(), Error>
    where
        Content: Into<Self::Content> + Send + 'static;
}
// NIT: Name TBD..? Reflog seems a bit exclusive.
#[async_trait]
pub trait ReflogStorage: Send + Sync {
    async fn list<P>(&self, path: P) -> Result<bool, Error>
    where
        P: AsRef<Path> + Send;
    // async fn exists<S>(&self, path: &[S]) -> Result<bool, Error>
    // where
    //     S: AsRef<str> + Send + Sync;
}
// TODO: Don't clobber Path[Buf]
pub struct PathBuf {}
pub enum SegmentBuf<Buf = Vec<u8>>
where
    Buf: Deref<Target = [u8]> + 'static,
{
    /// A non-UTF8 series of bytes as a segment, commonly a
    /// key fingerprint identifying a user to be used as a path
    /// segment.
    Buf(Buf),
    String(String),
}
// TODO: Don't clobber Path[Buf]
pub struct Path {}
pub enum Segment<'a> {
    /// A non-UTF8 series of bytes as a segment, commonly a
    /// series of bytes identifying a user to be used as a path
    /// segment.
    BufSlice(&'a [u8]),
    Str(&'a str),
}
