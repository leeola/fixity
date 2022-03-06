use {super::Error, crate::storage::MutStorage, async_trait::async_trait};

#[async_trait]
pub trait Meta<Rid, Cid>: Send + Sync
where
    Rid: Send + Sync + 'static,
    Cid: Send + Sync + 'static,
{
    async fn repos(&self, remote: &str) -> Result<Vec<String>, Error>;
    async fn branches(&self, remote: &str, repo: &str) -> Result<Vec<String>, Error>;
    async fn heads(&self, remote: &str, repo: &str, branch: &str)
        -> Result<Vec<(Rid, Cid)>, Error>;
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Rid,
    ) -> Result<Cid, Error>;
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        head: Cid,
    ) -> Result<(), Error>;
    async fn detatch_head(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        head: Cid,
    ) -> Result<(), Error>;
    async fn append_log(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        head: Cid,
        message: &str,
    ) -> Result<(), Error>;
    async fn logs(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Log<Rid, Cid>>, Error>;
}
#[derive(Debug)]
pub struct Log<Rid, Cid> {
    pub remote: String,
    pub repo: String,
    pub replica: Rid,
    pub head: Cid,
    pub message: String,
}
#[async_trait]
impl<T, Rid, Cid> Meta<Rid, Cid> for T
where
    T: MutStorage,
    Rid: Send + Sync + 'static,
    Cid: Send + Sync + 'static,
{
    async fn repos(&self, remote: &str) -> Result<Vec<String>, Error> {
        todo!()
    }
    async fn branches(&self, remote: &str, repo: &str) -> Result<Vec<String>, Error> {
        todo!()
    }
    async fn heads(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Vec<(Rid, Cid)>, Error> {
        todo!()
    }
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Rid,
    ) -> Result<Cid, Error> {
        todo!()
    }
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        head: Cid,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn detatch_head(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        head: Cid,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn append_log(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        head: Cid,
        message: &str,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn logs(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Log<Rid, Cid>>, Error> {
        todo!()
    }
}
