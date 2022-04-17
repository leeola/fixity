use {async_trait::async_trait, std::str};
type Error = ();
// TODO: Finalize this design. This is just a rough draft..
// even for this codebase, hah.
#[async_trait]
pub trait ObjectRefKv<V>: Send + Sync
where
    V: AsRef<[u8]> + Send + 'static,
{
    async fn exists<K>(&self, k: &[K]) -> Result<bool, Error>
    where
        K: AsRef<str> + Send + Sync;
    async fn read<K>(&self, k: &[K]) -> Result<V, Error>
    where
        K: AsRef<str> + Send + Sync;
    async fn write<K>(&self, k: Vec<K>, v: V) -> Result<(), Error>
    where
        K: AsRef<str> + Send + Sync + 'static;
    async fn list<K>(&self, k: &[K]) -> Result<Vec<String>, Error>
    where
        K: AsRef<str> + Send + Sync;
    async fn read_string<K>(&self, k: &[K]) -> Result<String, Error>
    where
        K: AsRef<str> + Send + Sync,
    {
        let buf = self.read(k).await?;
        let s = str::from_utf8(&buf.as_ref()).map_err(|_| ())?.to_owned();
        Ok(s)
    }
}
