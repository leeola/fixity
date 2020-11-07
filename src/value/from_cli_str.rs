use super::{Key, Path, Scalar, Value};

impl Scalar {
    /// An experimental implementation to parse a [`Scalar`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        todo!("Scalar from cli str")
    }
}
impl Value {
    /// An experimental implementation to parse a [`Value`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        todo!("Value from cli str")
    }
}
impl Key {
    /// An experimental implementation to parse a [`Key`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        todo!("Key from cli str")
    }
}
impl Path {
    /// An experimental implementation to parse a [`Path`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        todo!("Path from cli str")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid u32: `{0}`")]
    InvalidUint32(String),
}
