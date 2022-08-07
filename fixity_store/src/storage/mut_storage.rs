use super::Error;
use async_trait::async_trait;
use std::{ops::Deref, sync::Arc};

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
#[async_trait]
impl<T, U> MutStorage for T
where
    T: Deref<Target = U> + Send + Sync,
    U: MutStorage,
{
    type Value = U::Value;
    async fn list<K, D>(&self, prefix: K, delimiter: Option<D>) -> Result<Vec<String>, Error>
    where
        K: AsRef<str> + Send,
        D: AsRef<str> + Send,
    {
        self.deref().list(prefix, delimiter).await
    }
    async fn get<K>(&self, key: K) -> Result<Self::Value, Error>
    where
        K: AsRef<str> + Send,
    {
        self.deref().get(key).await
    }
    async fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<str> + Into<String> + Send,
        V: AsRef<[u8]> + Into<Vec<u8>> + Send,
    {
        self.deref().put(key, value).await
    }
}
#[cfg(test)]
pub mod test {
    use crate::storage::{memory::Memory, MutStorage};
    use rstest::*;
    #[rstest]
    #[case::test_storage(Memory::<()>::default())]
    #[tokio::test]
    async fn usage<M: MutStorage>(#[case] m: M) {
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
