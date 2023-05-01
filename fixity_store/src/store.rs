// pub mod json_store;
// pub mod rkyv_store;

use crate::storage::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("resource not found")]
    NotFound,
    #[error("resource not modified")]
    NotModified,
    // TODO: move to merge error type.
    #[error("type cannot be merged")]
    UnmergableType,
    // TODO: move to diff error type.
    #[error("type cannot be diff'd")]
    UndiffableType,
    #[error("storage: {0}")]
    Storage(StorageError),
}
impl From<StorageError> for StoreError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound => Self::NotFound,
            err => Self::Storage(err),
        }
    }
}
