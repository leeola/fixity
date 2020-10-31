use {
    crate::{deser, storage::Error as StorageError, value::Addr},
    std::io,
};
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{0}`")]
    Unhandled(String),
    #[error("builder error: `{message}`")]
    Builder { message: String },
    #[error("prolly error: `{message}`")]
    Prolly { message: String },
    #[error("prolly@`{addr}`, error: `{message}`")]
    ProllyAddr { addr: Addr, message: String },
    #[error("store error: `{0}`")]
    Storage(#[from] StorageError),
    #[error("io error: `{0}`")]
    Io(#[from] io::Error),
    #[error("reading input error: `{err}`")]
    IoInputRead { err: io::Error },
    #[error(
        "storage wrote {got} bytes,
        but was expected to write {expected} bytes"
    )]
    IncompleteWrite { got: usize, expected: usize },
    #[error("deser error: `{0}`")]
    Deser(#[from] deser::Error),
    #[cfg(feature = "serde_json")]
    #[error("serde json error: `{0}`")]
    SerdeJson(#[from] serde_json::error::Error),
    /// A Borsh error..
    ///
    /// for some reason they return an io::Error, the std::io type is not a bug.
    #[cfg(feature = "borsh")]
    #[error("borsh error: `{0:?}`")]
    Borsh(std::io::Error),
    /// A Borsh error, with an address..
    ///
    /// for some reason they return an io::Error, the std::io type is not a bug.
    #[cfg(feature = "borsh")]
    #[error("addr:{addr}, borsh error: `{err:?}`")]
    BorshAddr { addr: Addr, err: std::io::Error },
    #[cfg(feature = "cjson")]
    #[error("cjson error: `{0:?}`")]
    Cjson(cjson::Error),
}
#[cfg(feature = "cjson")]
impl From<cjson::Error> for Error {
    fn from(err: cjson::Error) -> Self {
        Self::Cjson(err)
    }
}
