use std::io;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{0}`")]
    Unhandled(String),
    #[error("store error: `{0}`")]
    StoreError(#[from] StoreError),
}
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("hash: `{hash}`, storage error: `{message}`")]
    Storage { hash: String, message: String },
    #[error("hash `{hash}` not found")]
    NotFound { hash: String },
    #[error("hash `{hash}` io error: {err}")]
    Io { hash: String, err: io::Error },
    #[error("hash `{hash}` io error: {err}")]
    Utf8 {
        hash: String,
        err: std::str::Utf8Error,
    },
}
