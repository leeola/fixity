use {async_trait::async_trait, std::str};
type Error = ();
#[async_trait]
pub trait ObjectKv<K, V>: Send + Sync
where
    K: AsRef<[u8]> + Send + Sync + 'static,
    V: AsRef<[u8]> + Send + 'static,
{
    async fn exists(&self, k: &K) -> Result<bool, Error>;
    async fn read(&self, k: &K) -> Result<V, Error>;
    async fn write(&self, k: K, v: V) -> Result<(), Error>;
    async fn read_string(&self, k: &K) -> Result<String, Error> {
        let buf = self.read(k).await?;
        let s = str::from_utf8(&buf.as_ref()).map_err(|_| ())?.to_owned();
        Ok(s)
    }
}
