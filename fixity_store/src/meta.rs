use crate::{
    contentid::ContentId,
    replicaid::{ReplicaId, Rid},
    storage::{MutStorage, StorageError},
};
use async_trait::async_trait;
use multibase::Base;
use thiserror::Error;

#[async_trait]
pub trait Meta<Cid>: Send + Sync
where
    Cid: Send + Sync + 'static,
{
    type Rid: ReplicaId + 'static;
    async fn repos(&self, remote: &str) -> Result<Vec<String>, MetaStoreError<Self::Rid, Cid>>;
    async fn branches(
        &self,
        remote: &str,
        repo: &str,
    ) -> Result<Vec<String>, MetaStoreError<Self::Rid, Cid>>;
    async fn heads(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Vec<(Self::Rid, Cid)>, MetaStoreError<Self::Rid, Cid>>;
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Self::Rid,
    ) -> Result<Cid, MetaStoreError<Self::Rid, Cid>>;
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: Self::Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Self::Rid, Cid>>;
    async fn append_log<S: Into<String> + Send>(
        &self,
        remote: &str,
        repo: &str,
        replica: Self::Rid,
        message: S,
    ) -> Result<(), MetaStoreError<Self::Rid, Cid>>;
    async fn logs(
        &self,
        remote: &str,
        repo: &str,
        replica: Self::Rid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Log<Self::Rid, Cid>>, MetaStoreError<Self::Rid, Cid>>;
}
#[derive(Error, Debug)]
pub enum MetaStoreError<Rid, Cid> {
    #[error("resource not found")]
    NotFound,
    #[error("invalid content id: {message}")]
    InvalidCid { cid: Cid, message: Box<str> },
    #[error("invalid replica id: {message}")]
    InvalidRid { message: Box<str> },
    #[error("storage {}/{}/{}, : {error}",
        .remote.unwrap_or(Box::from("")),
        .repo.unwrap_or(Box::from("")),
        .branch.unwrap_or(Box::from("")),
        // .rid.unwrap_or(Box::from("")),
    )]
    Storage {
        remote: Option<Box<str>>,
        repo: Option<Box<str>>,
        branch: Option<Box<str>>,
        rid: Option<Rid>,
        cid: Option<Cid>,
        error: StorageError,
    },
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
impl<T, Cid> Meta<Cid> for T
where
    T: MutStorage,
    Cid: ContentId + 'static,
{
    type Rid = Rid<8>;
    async fn repos(&self, remote: &str) -> Result<Vec<String>, MetaStoreError<Self::Rid, Cid>> {
        let dirs = list_dirs(self, format!("{remote}/"))
            .await
            .map_err(|error| MetaStoreError::Storage {
                remote: Some(Box::from(remote)),
                repo: None,
                branch: None,
                rid: None,
                cid: None,
                error,
            })?;
        Ok(dirs)
    }
    async fn branches(
        &self,
        remote: &str,
        repo: &str,
    ) -> Result<Vec<String>, MetaStoreError<Self::Rid, Cid>> {
        let dirs = list_dirs(self, format!("{remote}/{repo}/"))
            .await
            .map_err(|error| MetaStoreError::Storage {
                remote: Some(Box::from(remote)),
                repo: Some(Box::from(repo)),
                branch: None,
                rid: None,
                cid: None,
                error,
            })?;
        Ok(dirs)
    }
    async fn heads(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Vec<(Self::Rid, Cid)>, MetaStoreError<Self::Rid, Cid>> {
        let branch_path = format!("{remote}/{repo}/{branch}/");
        let paths = self
            .list::<_, &str>(&branch_path, None)
            .await
            .map_err(|error| MetaStoreError::Storage {
                remote: Some(Box::from(remote)),
                repo: Some(Box::from(repo)),
                branch: Some(Box::from(branch)),
                rid: None,
                cid: None,
                error,
            })?;
        let mut items = Vec::new();
        for path in paths {
            let encoded_rid = path.strip_prefix(&branch_path).unwrap();
            let (_, rid_bytes) =
                multibase::decode(&encoded_rid).map_err(|err| MetaStoreError::InvalidRid {
                    message: format!("decoding rid: {}", err).into_boxed_str(),
                })?;
            let rid =
                Self::Rid::from_hash(rid_bytes).ok_or_else(|| MetaStoreError::InvalidRid {
                    message: Box::from("creating rid"),
                })?;
            let cid_value = self.get(&path).await.unwrap();
            let encoded_cid = std::str::from_utf8(cid_value.as_ref())
                .map_err(|_| ())
                .unwrap();
            let (_, head_bytes) = multibase::decode(encoded_cid).map_err(|_| ()).unwrap();
            let head = Cid::from_hash(head_bytes).unwrap();
            items.push((rid, head));
        }
        Ok(items)
    }
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Self::Rid,
    ) -> Result<Cid, MetaStoreError<Self::Rid, Cid>> {
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
        replica: Self::Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Self::Rid, Cid>> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, replica.as_bytes());
        let head = multibase::encode(MUT_CID_RID_ENCODING, head.as_bytes());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        self.put(path, head).await
    }
    async fn append_log<S: Into<String> + Send>(
        &self,
        _remote: &str,
        _repo: &str,
        _replica: Self::Rid,
        _message: S,
    ) -> Result<(), MetaStoreError<Self::Rid, Cid>> {
        todo!()
    }
    async fn logs(
        &self,
        _remote: &str,
        _repo: &str,
        _replica: Self::Rid,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<Log<Self::Rid, Cid>>, MetaStoreError<Self::Rid, Cid>> {
        todo!()
    }
}
async fn list_dirs<S: MutStorage>(
    storage: &S,
    base_path: String,
) -> Result<Vec<String>, StorageError> {
    let paths = storage.list::<_, &str>(&base_path, Some("/")).await?;
    let dirs = paths
        .into_iter()
        .filter_map(|path| {
            // silently dropping items in the repo that may not be great, but we can't
            // fail either since users could make the repo dirty. So in general, ignore.
            let mut dir = path.strip_prefix(&base_path)?.to_owned();
            // each dir ends in a delim, so drop it.
            let _ = dir.pop();
            Some(dir)
        })
        .collect::<Vec<_>>();
    Ok(dirs)
}
#[cfg(test)]
pub mod meta_mut_storage {
    use crate::{storage::Memory, Meta};

    use rstest::*;
    #[rstest]
    #[case::test_storage(Memory::<[u8; 4]>::default())]
    #[tokio::test]
    async fn basic<S: Meta<[u8; 4], Rid = [u8; 8]>>(#[case] s: S) {
        let rida = [10u8; 8];
        let ridb = [20u8; 8];
        s.set_head("remote", "repo", "branch", rida, [1u8; 4])
            .await
            .unwrap();
        s.set_head("remote", "repo", "branch", ridb, [2u8; 4])
            .await
            .unwrap();
        assert_eq!(
            s.head("remote", "repo", "branch", &rida).await.unwrap(),
            [1u8; 4],
        );
        assert_eq!(
            s.head("remote", "repo", "branch", &ridb).await.unwrap(),
            [2u8; 4],
        );
        assert_eq!(
            s.heads("remote", "repo", "branch").await.unwrap(),
            vec![([10u8; 8], [1u8; 4]), ([20u8; 8], [2u8; 4])],
        );
        s.set_head("remote", "repo", "foo", ridb, [2u8; 4])
            .await
            .unwrap();
        assert_eq!(
            s.branches("remote", "repo").await.unwrap(),
            vec!["branch", "foo"]
        );
        assert_eq!(s.repos("remote").await.unwrap(), vec!["repo"]);
    }
}