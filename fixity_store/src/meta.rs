use crate::{
    contentid::NewContentId,
    replicaid::{NewReplicaId, Rid},
    storage::{MutStorage, StorageError},
};
use async_trait::async_trait;
use multibase::Base;
use std::fmt::Display;
use thiserror::Error;

#[async_trait]
pub trait Meta<Cid>: Send + Sync
where
    Cid: Send + Sync + 'static,
{
    type Rid: NewReplicaId + 'static;
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
        replica: &Self::Rid,
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
async fn get_cid_from_path<MS: MutStorage, Rid: NewReplicaId, Cid: NewContentId>(
    ms: &MS,
    remote: &str,
    repo: &str,
    branch: &str,
    rid: &Rid,
    path: &str,
) -> Result<Cid, MetaStoreError<Rid, Cid>> {
    let cid_value = ms.get(&path).await.map_err(|err| match err {
        StorageError::NotFound => MetaStoreError::NotFound,
        err => MetaStoreError::Storage {
            remote: Some(Box::from(remote)),
            repo: Some(Box::from(repo)),
            branch: Some(Box::from(branch)),
            rid: Some(rid.clone()),
            cid: None,
            err,
        },
    })?;
    let encoded_cid =
        std::str::from_utf8(cid_value.as_ref()).map_err(|err| MetaStoreError::Cid {
            remote: Some(Box::from(remote)),
            repo: Some(Box::from(repo)),
            branch: Some(Box::from(branch)),
            rid: Some(rid.clone()),
            message: format!("verifying cid utf8: {}", err).into_boxed_str(),
        })?;
    let (_, head_bytes) = multibase::decode(encoded_cid).map_err(|err| MetaStoreError::Cid {
        remote: Some(Box::from(remote)),
        repo: Some(Box::from(repo)),
        branch: Some(Box::from(branch)),
        rid: Some(rid.clone()),
        message: format!("decoding head cid: {}", err).into_boxed_str(),
    })?;
    Cid::from_hash(head_bytes).map_err(|_| MetaStoreError::Cid {
        remote: Some(Box::from(remote)),
        repo: Some(Box::from(repo)),
        branch: Some(Box::from(branch)),
        rid: Some(rid.clone()),
        message: Box::from("creating cid from head bytes"),
    })
}
#[derive(Error, Debug)]
pub enum MetaStoreError<Rid, Cid> {
    #[error("resource not found")]
    NotFound,
    #[error("invalid replica id: {message}")]
    Rid {
        remote: Option<Box<str>>,
        repo: Option<Box<str>>,
        branch: Option<Box<str>>,
        // TODO: convert to rid error.
        message: Box<str>,
    },
    #[error("cid: {message}")]
    Cid {
        remote: Option<Box<str>>,
        repo: Option<Box<str>>,
        branch: Option<Box<str>>,
        rid: Option<Rid>,
        // TODO: convert to cid error.
        message: Box<str>,
    },
    #[error("storage {}/{}/{}, : {err}",
        .remote.clone().unwrap_or(Box::from("")),
        .repo.clone().unwrap_or(Box::from("")),
        .branch.clone().unwrap_or(Box::from("")),
        // .rid.unwrap_or(Box::from("")),
    )]
    Storage {
        remote: Option<Box<str>>,
        repo: Option<Box<str>>,
        branch: Option<Box<str>>,
        rid: Option<Rid>,
        cid: Option<Cid>,
        err: StorageError,
    },
    #[error("{}/{}/{}, {}: {message}",
        .remote.clone().unwrap_or(Box::from("")),
        // .repo.unwrap_or(Box::from("")),
        DisplayOption(.repo),
        .branch.clone().unwrap_or(Box::from("")),
        // .rid.map(|rid| format!("{}"))
        DisplayOption(.repo)
    )]
    Other {
        remote: Option<Box<str>>,
        repo: Option<Box<str>>,
        branch: Option<Box<str>>,
        rid: Option<Rid>,
        cid: Option<Cid>,
        message: Box<str>,
    },
}
#[derive(Debug)]
struct DisplayOption<'a, T: Display>(pub &'a Option<T>);
impl<T> Display for DisplayOption<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(t) = self.0.as_ref() {
            write!(f, "{}", t)
        } else {
            Ok(())
        }
    }
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
    Cid: NewContentId + 'static,
{
    type Rid = Rid<8>;
    async fn repos(&self, remote: &str) -> Result<Vec<String>, MetaStoreError<Self::Rid, Cid>> {
        let dirs =
            list_dirs(self, format!("{remote}/"))
                .await
                .map_err(|err| MetaStoreError::Storage {
                    remote: Some(Box::from(remote)),
                    repo: None,
                    branch: None,
                    rid: None,
                    cid: None,
                    err,
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
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(Box::from(remote)),
                repo: Some(Box::from(repo)),
                branch: None,
                rid: None,
                cid: None,
                err,
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
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(Box::from(remote)),
                repo: Some(Box::from(repo)),
                branch: Some(Box::from(branch)),
                rid: None,
                cid: None,
                err,
            })?;
        let mut items = Vec::new();
        for path in paths {
            let encoded_rid = path.strip_prefix(&branch_path).unwrap();
            let (_, rid_bytes) =
                multibase::decode(&encoded_rid).map_err(|err| MetaStoreError::Rid {
                    remote: Some(Box::from(remote)),
                    repo: Some(Box::from(repo)),
                    branch: Some(Box::from(branch)),
                    message: format!("decoding rid: {}", err).into_boxed_str(),
                })?;
            let rid = Self::Rid::from_buf(rid_bytes).map_err(|_| MetaStoreError::Rid {
                remote: Some(Box::from(remote)),
                repo: Some(Box::from(repo)),
                branch: Some(Box::from(branch)),
                message: Box::from("creating rid"),
            })?;
            let head = get_cid_from_path(self, remote, repo, branch, &rid, &path).await?;
            items.push((rid, head));
        }
        Ok(items)
    }
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        rid: &Self::Rid,
    ) -> Result<Cid, MetaStoreError<Self::Rid, Cid>> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        get_cid_from_path(self, remote, repo, branch, rid, &path).await
    }
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        rid: &Self::Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Self::Rid, Cid>> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
        let head = multibase::encode(MUT_CID_RID_ENCODING, head.as_hash());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        self.put(path, head)
            .await
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(Box::from(remote)),
                repo: Some(Box::from(repo)),
                branch: Some(Box::from(branch)),
                rid: Some(rid.clone()),
                cid: None,
                err,
            })?;
        Ok(())
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
    use crate::{contentid::Cid, replicaid::Rid, storage::Memory, Meta};

    use rstest::*;
    #[rstest]
    #[case::test_storage(Memory::<Cid<4>>::default())]
    #[tokio::test]
    async fn basic<S: Meta<Cid, Rid = Rid<8>>>(#[case] s: S) {
        let rida = Rid::from([10u8; 8]);
        let ridb = Rid::from([20u8; 8]);
        s.set_head("remote", "repo", "branch", &rida, [1u8; 4].into())
            .await
            .unwrap();
        s.set_head("remote", "repo", "branch", &ridb, [2u8; 4].into())
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
            vec![
                ([10u8; 8].into(), [1u8; 4].into()),
                ([20u8; 8].into(), [2u8; 4].into())
            ],
        );
        s.set_head("remote", "repo", "foo", &ridb, [2u8; 4].into())
            .await
            .unwrap();
        assert_eq!(
            s.branches("remote", "repo").await.unwrap(),
            vec!["branch", "foo"]
        );
        assert_eq!(s.repos("remote").await.unwrap(), vec!["repo"]);
    }
}
