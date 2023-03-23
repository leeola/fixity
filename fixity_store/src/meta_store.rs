use std::fmt::Display;

use crate::{
    contentid::NewContentId,
    mut_store::{MutStore, MutStoreError},
    replicaid::NewReplicaId,
};
use async_trait::async_trait;
use multibase::Base;
use thiserror::Error;

#[async_trait]
pub trait MetaStore<Rid: NewReplicaId, Cid: NewContentId>: Send + Sync {
    /// List all Replicas under a specific Remote.
    async fn replicas(&self, remote: &str) -> Result<Vec<Rid>, MetaStoreError<Rid, Cid>>;
    /// Get the head for the given Replica.
    async fn head(&self, remote: &str, rid: &Rid) -> Result<Cid, MetaStoreError<Rid, Cid>>;
    /// List the heads for the provided Replicas.
    async fn heads(
        &self,
        remote: &str,
        rids: &[Rid],
    ) -> Result<Vec<(Rid, Cid)>, MetaStoreError<Rid, Cid>>;
    async fn set_head(
        &self,
        remote: &str,
        rid: &Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Rid, Cid>>;
    // Not sure if i want to keep this. Need to handle config storage somewhere, but syncing
    // it to remotes feels wrong.
    //
    // async fn set_remote_config(
    //     &self,
    //     remote: &str,
    //     config: RemoteConfig,
    // ) -> Result<(), MetaStoreError<Rid, Cid>>;
}
async fn get_cid_from_path<MS: MutStore, Rid: NewReplicaId, Cid: NewContentId>(
    ms: &MS,
    remote: &str,
    rid: &Rid,
    path: &str,
) -> Result<Cid, MetaStoreError<Rid, Cid>> {
    let cid_value = ms.get(&path).await.map_err(|err| match err {
        MutStoreError::NotFound => MetaStoreError::NotFound,
        err => MetaStoreError::Storage {
            remote: Some(String::from(remote)),
            repo: None,
            branch: None,
            rid: Some(rid.clone()),
            cid: None,
            err,
        },
    })?;
    let encoded_cid =
        std::str::from_utf8(cid_value.as_ref()).map_err(|err| MetaStoreError::Cid {
            remote: Some(String::from(remote)),
            repo: None,
            branch: None,
            rid: Some(rid.clone()),
            message: format!("verifying cid utf8: {}", err),
        })?;
    let (_, head_bytes) = multibase::decode(encoded_cid).map_err(|err| MetaStoreError::Cid {
        remote: Some(String::from(remote)),
        repo: None,
        branch: None,
        rid: Some(rid.clone()),
        message: format!("decoding head cid: {}", err),
    })?;
    Cid::from_hash(head_bytes).map_err(|_| MetaStoreError::Cid {
        remote: Some(String::from(remote)),
        repo: None,
        branch: None,
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
    async fn replicas(&self, remote: &str) -> Result<Vec<Rid>, MetaStoreError<Rid, Cid>> {
        let remote_path = format!("{remote}/");
        let paths = self.list::<_, &str>(&remote_path, None).await.unwrap();
        let mut items = Vec::new();
        for path in paths {
            let encoded_rid = path.strip_prefix(&remote_path).unwrap();
            let (_, rid_bytes) = multibase::decode(&encoded_rid).unwrap();
            let rid = Rid::from_buf(rid_bytes).unwrap();
            items.push(rid);
        }
        Ok(items)
    }
    async fn head(&self, remote: &str, rid: &Rid) -> Result<Cid, MetaStoreError<Rid, Cid>> {
        let encoded_rid = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
        let path = format!("{remote}/{encoded_rid}");
        let (_, rid_bytes) =
            multibase::decode(&encoded_rid).map_err(|err| MetaStoreError::Rid {
                remote: Some(String::from(remote)),
                repo: None,
                branch: None,
                message: format!("decoding rid: {}", err),
            })?;
        let head = get_cid_from_path(self, remote, rid, &path).await?;
        Ok(head)
    }
    async fn heads(
        &self,
        remote: &str,
        replicas: &[Rid],
    ) -> Result<Vec<(Rid, Cid)>, MetaStoreError<Rid, Cid>> {
        let mut heads = Vec::with_capacity(replicas.len());
        for rid in replicas {
            let encoded_rid = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
            let path = format!("{remote}/{encoded_rid}");
            let (_, rid_bytes) =
                multibase::decode(&encoded_rid).map_err(|err| MetaStoreError::Rid {
                    remote: Some(String::from(remote)),
                    repo: None,
                    branch: None,
                    message: format!("decoding rid: {}", err),
                })?;
            let head: Cid = get_cid_from_path(self, remote, rid, &path).await?;
            heads.push((rid.clone(), head));
        }
        Ok(heads)
    }
    async fn set_head(
        &self,
        remote: &str,
        rid: &Rid,
        head: Cid,
    ) -> Result<(), MetaStoreError<Rid, Cid>> {
        let encoded_rid = multibase::encode(MUT_CID_RID_ENCODING, rid.as_buf());
        let encoded_head = multibase::encode(MUT_CID_RID_ENCODING, head.as_hash());
        let path = format!("{remote}/{encoded_rid}");
        self.put(path, encoded_head)
            .await
            .map_err(|err| MetaStoreError::Storage {
                remote: Some(String::from(remote)),
                repo: None,
                branch: None,
                rid: Some(rid.clone()),
                cid: None,
                err,
            })?;
        Ok(())
    }
}
#[cfg(test)]
pub mod meta_mut_storage {
    use super::*;
    use crate::{contentid::Cid, replicaid::Rid, storage::Memory};
    use rstest::*;

    #[rstest]
    #[case::test_storage(Memory::<_>::default())]
    #[tokio::test]
    async fn basic<S: MetaStore<i32, i32>>(#[case] s: S) {
        s.set_head("remote", &1, 10).await.unwrap();
        s.set_head("remote", &2, 20).await.unwrap();
        assert_eq!(s.head("remote", &1).await.unwrap(), 10);
        assert_eq!(s.head("remote", &2).await.unwrap(), 20);
        assert_eq!(
            s.heads("remote", &[1, 2]).await.unwrap(),
            vec![(1, 10), (20, 20)],
        );
        s.set_head("remote", &2, 20).await.unwrap();
    }
}
