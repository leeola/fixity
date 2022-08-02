use super::Error;
use crate::{cid::ContentId, rid::ReplicaId, storage::MutStorage};
use async_trait::async_trait;
use multibase::Base;

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
        branch: &str,
        replica: Rid,
        head: Cid,
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
const MUT_CID_RID_ENCODING: Base = Base::Base32HexLower;
// NIT: This should probably be a wrapper type, rather than a blanket
// impl. As the blanket impl is focused on implicitly being filesystem
// and this seems wrong to assume in all cases.
//
// A simple `MetaOverMut(pub T)` wrapper struct would help make this
// explicit.
//
// IMPORTANT: This impl is not using any kind of escaping, so `/`
// breaks the whole thing at the moment. POC impl, beware.
#[async_trait]
impl<T, Rid, Cid> Meta<Rid, Cid> for T
where
    T: MutStorage,
    Rid: ReplicaId + 'static,
    Cid: ContentId + 'static,
{
    async fn repos(&self, _remote: &str) -> Result<Vec<String>, Error> {
        todo!()
    }
    async fn branches(&self, _remote: &str, _repo: &str) -> Result<Vec<String>, Error> {
        todo!()
    }
    async fn heads(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Vec<(Rid, Cid)>, Error> {
        let branch_path = format!("{remote}/{repo}/{branch}/");
        let paths = self.list(&branch_path).await?;
        let mut items = Vec::new();
        for path in paths {
            let replica_path = path.strip_prefix(&branch_path).unwrap();
            let (_, rid_bytes) = multibase::decode(&replica_path).map_err(|_| ())?;
            let rid = Rid::from_hash(rid_bytes)?;
            let cid_value = self.get(replica_path).await?;
            let encoded_cid = std::str::from_utf8(cid_value.as_ref()).map_err(|_| ())?;
            let (_, head_bytes) = multibase::decode(encoded_cid).map_err(|_| ())?;
            let head = Cid::from_hash(head_bytes)?;
            items.push((rid, head));
        }
        Ok(items)
    }
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Rid,
    ) -> Result<Cid, Error> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, replica.as_bytes());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        let value = self.get(path).await?;
        let encoded_cid = std::str::from_utf8(value.as_ref()).map_err(|_| ())?;
        let (_, head_bytes) = multibase::decode(encoded_cid).map_err(|_| ())?;
        let head = Cid::from_hash(head_bytes)?;
        Ok(head)
    }
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: Rid,
        head: Cid,
    ) -> Result<(), Error> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, replica.as_bytes());
        let head = multibase::encode(MUT_CID_RID_ENCODING, head.as_bytes());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        self.put(path, head).await
    }
    async fn logs(
        &self,
        _remote: &str,
        _repo: &str,
        _replica: Rid,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<Log<Rid, Cid>>, Error> {
        todo!()
    }
}
#[cfg(test)]
pub mod meta_mut_storage {
    use crate::{cid::CID_LENGTH, storage::Memory, Meta, MutStorage};
    use std::sync::Arc;

    use rstest::*;
    #[rstest]
    #[case::test_storage(Memory::<[u8; 4]>::default())]
    #[tokio::test]
    async fn basic<S: Meta<[u8; 4], [u8; 4]>>(#[case] s: S) {
        let rid = [1u8; 4];
        s.set_head("remote", "repo", "brancha", rid, [2u8; 4])
            .await
            .unwrap();
        s.set_head("remote", "repo", "branchb", rid, [3u8; 4])
            .await
            .unwrap();
        assert_eq!(
            s.head("remote", "repo", "brancha", &rid).await.unwrap(),
            [2u8; 4],
        );
        assert_eq!(
            s.head("remote", "repo", "branchb", &rid).await.unwrap(),
            [3u8; 4],
        );
    }
}
