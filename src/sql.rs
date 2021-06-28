/// Sql database related error variants.
#[derive(Debug, thiserror::Error)]
pub enum SqlError {
    #[error("unhandled error: `{0}`")]
    Unhandled(String),
}
