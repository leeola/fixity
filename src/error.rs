use {
    crate::{deser, fixity, storage, value::Addr, workspace},
    std::io,
};
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: `{0}`")]
    Unhandled(String),
    /// An internal error to Fixity where user action is not expected.
    #[error("fixity encountered an internal error: {source}")]
    Internal {
        #[from]
        source: InternalError,
    },
    /// A fixi repository was not found.
    #[error("fixity repository was not found")]
    RepositoryNotFound,
    /// An action was attempted that writes changes to the repository, but
    /// no changes exist.
    #[error("no changes to write to repository")]
    NoChangesToWrite,
    /// A commit was attempted without any changes staged.
    #[error("a commit was attempted without any changes to commit")]
    NoStageToCommit,
    /// An action is unsupported when the HEAD is detached.
    #[error("an action is unsupported when the HEAD is detached")]
    DetachedHead,
    /// Writing a non-[`Map`](crate::Map) to the root of the Fixity repository is
    /// not allowed in most cases, as it would dangle the majority of pointers.
    #[error("cannot replace root data structure with non-map")]
    CannotReplaceRootMap,
    /// An addr exists but data was not found in storage.
    #[error("an address if dangling: `{message}`")]
    DanglingAddr {
        message: String,
        /// The address that is dangling, if available.
        ///
        /// The optional address in the error allows the caller to provide it if available,
        /// but not allocate prematurely for the error condition.
        addr: Option<Addr>,
    },
    #[error("data type error: {0}")]
    Type(#[from] TypeError),
    #[error("builder error: `{message}`")]
    Builder { message: String },
    #[error("prolly error: `{message}`")]
    Prolly { message: String },
    #[error("prolly@`{addr}`, error: `{message}`")]
    ProllyAddr { addr: Addr, message: String },
    #[error("store error: `{0}`")]
    Storage(#[from] storage::Error),
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
#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("fixity failed to initialize a new repository: {source}")]
    Init {
        #[from]
        source: fixity::InitError,
    },
    #[error("head: `{source}`")]
    Workspace {
        #[from]
        source: workspace::Error,
    },
    #[error("primitive: `{0}`")]
    Primitive(String),
    #[error("path: `{0}`")]
    Path(String),
    /// An internal input/output error.
    #[error("input/output error: `{0}`")]
    Io(String),
}
#[cfg(feature = "cjson")]
impl From<cjson::Error> for Error {
    fn from(err: cjson::Error) -> Self {
        Self::Cjson(err)
    }
}
impl From<workspace::Error> for Error {
    fn from(err: workspace::Error) -> Self {
        Self::Internal { source: err.into() }
    }
}
impl From<fixity::InitError> for Error {
    fn from(err: fixity::InitError) -> Self {
        Self::Internal { source: err.into() }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    #[error("expected a Value of a specific type, got another")]
    UnexpectedValueVariant {
        /// The segment of the error within a [`crate::Path`], if available.
        at_segment: Option<String>,
        /// The address of the error, if available.
        at_addr: Option<Addr>,
    },
}
