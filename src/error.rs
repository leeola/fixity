use crate::storage::Error as StorageError;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{0}`")]
    Unhandled(String),
    #[error("store error: `{0}`")]
    StorageError(#[from] StorageError),
}
