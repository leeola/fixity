pub mod memory;
use async_trait::async_trait;
pub use memory::Memory;
use std::{str, sync::Arc};
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
    async fn write_unchecked<B>(&self, cid: Cid, bytes: B) -> Result<(), Error>
    where
        B: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
}
#[async_trait]
pub trait MutStorage: Send + Sync {
    type Value: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn list<K, D>(&self, prefix: K, delimiter: Option<D>) -> Result<Vec<String>, Error>
    where
        K: AsRef<str> + Send,
        D: AsRef<str> + Send;
    async fn get<K>(&self, key: K) -> Result<Self::Value, Error>
    where
        K: AsRef<str> + Send;
    async fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<str> + Into<String> + Send,
        V: AsRef<[u8]> + Into<Vec<u8>> + Send;
}
// pub struct Path {
#[cfg(test)]
pub mod test {
    use super::{memory::Memory, MutStorage};
    use rstest::*;
    #[rstest]
    #[case::test_storage(Memory::<()>::default())]
    #[tokio::test]
    async fn mut_storage<M: MutStorage>(#[case] m: M) {
        m.put("foo", "bar").await.unwrap();
        assert_eq!(m.get("foo").await.unwrap().as_ref(), b"bar");
        m.put("foo/bar", "boo").await.unwrap();
        m.put("zaz", "zoinks").await.unwrap();
        assert_eq!(
            m.list::<_, &str>("foo", None).await.unwrap(),
            vec!["foo".to_string(), "foo/bar".to_string()]
        );
        assert!(m.list::<_, &str>("b", None).await.unwrap().is_empty());
    }
}
