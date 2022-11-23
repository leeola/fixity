use std::fmt::Display;

use crate::{
    contentid::NewContentId,
    mut_store::{MutStore, MutStoreError},
    replicaid::NewReplicaId,
    storage::StorageError,
};
use async_trait::async_trait;
use multibase::Base;
use thiserror::Error;

#[async_trait]
pub trait MetaStore<Rid: NewReplicaId, Cid: NewContentId>: Send + Sync {
    async fn repos(&self, remote: &str) -> Result<Vec<String>, MetaStoreError<Rid, Cid>>;
    async fn branches(
        &self,
        remote: &str,
        repo: &str,
    ) -> Result<Vec<String>, MetaStoreError<Rid, Cid>>;
    async fn heads(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Vec<(Rid, Cid)>, MetaStoreError<Rid, Cid>>;
    async fn head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Rid,
    ) -> Result<Cid, MetaStoreError<Rid, Cid>>;
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        replica: &Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Rid, Cid>>;
    async fn append_log<S: Into<String> + Send>(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        message: S,
    ) -> Result<(), MetaStoreError<Rid, Cid>>;
    async fn logs(
        &self,
        remote: &str,
        repo: &str,
        replica: Rid,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Log<Rid, Cid>>, MetaStoreError<Rid, Cid>>;
}
async fn get_cid_from_path<MS: MutStore, Rid: NewReplicaId, Cid: NewContentId>(
    ms: &MS,
    remote: &str,
    repo: &str,
    branch: &str,
    rid: &Rid,
    path: &str,
) -> Result<Cid, MetaStoreError<Rid, Cid>> {
    let cid_value = ms.get(&path).await.map_err(|err| match err {
        MutStoreError::NotFound => MetaStoreError::NotFound,
        err => MetaStoreError::Storage {
            remote: Some(String::from(remote)),
            repo: Some(String::from(repo)),
            branch: Some(String::from(branch)),
            rid: Some(rid.clone()),
            cid: None,
            err,
        },
    })?;
    let encoded_cid =
        std::str::from_utf8(cid_value.as_ref()).map_err(|err| MetaStoreError::Cid {
            remote: Some(String::from(remote)),
            repo: Some(String::from(repo)),
            branch: Some(String::from(branch)),
            rid: Some(rid.clone()),
            message: format!("verifying cid utf8: {}", err),
        })?;
    let (_, head_bytes) = multibase::decode(encoded_cid).map_err(|err| MetaStoreError::Cid {
        remote: Some(String::from(remote)),
        repo: Some(String::from(repo)),
        branch: Some(String::from(branch)),
        rid: Some(rid.clone()),
        message: format!("decoding head cid: {}", err),
    })?;
    Cid::from_hash(head_bytes).map_err(|_| MetaStoreError::Cid {
        remote: Some(String::from(remote)),
        repo: Some(String::from(repo)),
        branch: Some(String::from(branch)),
        rid: Some(rid.clone()),
        message: String::from("creating cid from head bytes"),
    })
}
#[derive(Error, Debug)]
pub enum MetaStoreError<Rid, Cid> {
    #[error("resource not found")]
    NotFound,
    #[error("invalid replica id: {message}")]
    Rid {
        remote: Option<String>,
        repo: Option<String>,
        branch: Option<String>,
        // TODO: convert to rid error.
        message: String,
    },
    #[error("cid: {message}")]
    Cid {
        remote: Option<String>,
        repo: Option<String>,
        branch: Option<String>,
        rid: Option<Rid>,
        // TODO: convert to cid error.
        message: String,
    },
    #[error("storage {}/{}/{}, : {err}",
        .remote.clone().unwrap_or(String::from("")),
        .repo.clone().unwrap_or(String::from("")),
        .branch.clone().unwrap_or(String::from("")),
        // .rid.unwrap_or(String::from("")),
    )]
    Storage {
        remote: Option<String>,
        repo: Option<String>,
        branch: Option<String>,
        rid: Option<Rid>,
        cid: Option<Cid>,
        err: MutStoreError,
    },
    #[error("{}/{}/{}, {}: {message}",
        .remote.clone().unwrap_or(String::from("")),
        // .repo.unwrap_or(String::from("")),
        DisplayOption(.repo),
        .branch.clone().unwrap_or(String::from("")),
        // .rid.map(|rid| format!("{}"))
        DisplayOption(.repo)
    )]
    Other {
        remote: Option<String>,
        repo: Option<String>,
        branch: Option<String>,
        rid: Option<Rid>,
        cid: Option<Cid>,
        message: String,
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
impl<T, Rid, Cid> MetaStore<Rid, Cid> for T
where
    T: MutStore,
    Rid: NewReplicaId,
    Cid: NewContentId,
{
    async fn repos(&self, remote: &str) -> Result<Vec<String>, MetaStoreError<Rid, Cid>> {
        let dirs =
            list_dirs(self, format!("{remote}/"))
                .await
                .map_err(|err| MetaStoreError::Storage {
                    remote: Some(String::from(remote)),
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
    ) -> Result<Vec<String>, MetaStoreError<Rid, Cid>> {
        let dirs = list_dirs(self, format!("{remote}/{repo}/"))
            .await
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(String::from(remote)),
                repo: Some(String::from(repo)),
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
    ) -> Result<Vec<(Rid, Cid)>, MetaStoreError<Rid, Cid>> {
        let branch_path = format!("{remote}/{repo}/{branch}/");
        let paths = self
            .list::<_, &str>(&branch_path, None)
            .await
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(String::from(remote)),
                repo: Some(String::from(repo)),
                branch: Some(String::from(branch)),
                rid: None,
                cid: None,
                err,
            })?;
        let mut items = Vec::new();
        for path in paths {
            let encoded_rid = path.strip_prefix(&branch_path).unwrap();
            let (_, rid_bytes) =
                multibase::decode(&encoded_rid).map_err(|err| MetaStoreError::Rid {
                    remote: Some(String::from(remote)),
                    repo: Some(String::from(repo)),
                    branch: Some(String::from(branch)),
                    message: format!("decoding rid: {}", err),
                })?;
            let rid = Rid::from_buf(rid_bytes).map_err(|_| MetaStoreError::Rid {
                remote: Some(String::from(remote)),
                repo: Some(String::from(repo)),
                branch: Some(String::from(branch)),
                message: String::from("creating rid"),
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
        rid: &Rid,
    ) -> Result<Cid, MetaStoreError<Rid, Cid>> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        get_cid_from_path(self, remote, repo, branch, rid, &path).await
    }
    async fn set_head(
        &self,
        remote: &str,
        repo: &str,
        branch: &str,
        rid: &Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Rid, Cid>> {
        let replica = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
        let head = multibase::encode(MUT_CID_RID_ENCODING, head.as_hash());
        let path = format!("{remote}/{repo}/{branch}/{replica}");
        self.put(path, head)
            .await
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(String::from(remote)),
                repo: Some(String::from(repo)),
                branch: Some(String::from(branch)),
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
        _replica: Rid,
        _message: S,
    ) -> Result<(), MetaStoreError<Rid, Cid>> {
        todo!()
    }
    async fn logs(
        &self,
        _remote: &str,
        _repo: &str,
        _replica: Rid,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<Log<Rid, Cid>>, MetaStoreError<Rid, Cid>> {
        todo!()
    }
}
async fn list_dirs<S: MutStore>(
    storage: &S,
    base_path: String,
) -> Result<Vec<String>, MutStoreError> {
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
    async fn basic<S: Meta<Cid<4>, Rid = Rid<8>>>(#[case] s: S) {
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
