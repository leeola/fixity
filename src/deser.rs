#[cfg(all(feature = "deser_borsh", feature = "deser_json"))]
pub trait Serialize: borsh::BorshSerialize + serde::Serialize {}
#[cfg(all(feature = "deser_borsh", feature = "deser_json"))]
impl<T> Serialize for T where T: borsh::BorshSerialize + serde::Serialize {}
#[cfg(all(feature = "deser_borsh", feature = "deser_json"))]
pub trait Deserialize: borsh::BorshDeserialize + serde::de::DeserializeOwned {}
#[cfg(all(feature = "deser_borsh", feature = "deser_json"))]
impl<T> Deserialize for T where T: borsh::BorshDeserialize + serde::de::DeserializeOwned {}

#[cfg(all(feature = "deser_borsh", not(feature = "deser_json")))]
pub trait Serialize: borsh::BorshSerialize {}
#[cfg(all(feature = "deser_borsh", not(feature = "deser_json")))]
impl<T> Serialize for T where T: borsh::BorshSerialize {}
#[cfg(all(feature = "deser_borsh", not(feature = "deser_json")))]
pub trait Deserialize: borsh::BorshDeserialize {}
#[cfg(all(feature = "deser_borsh", not(feature = "deser_json")))]
impl<T> Deserialize for T where T: borsh::BorshDeserialize {}
///
#[derive(Debug, Copy, Clone)]
pub enum Deser {
    #[cfg(feature = "deser_borsh")]
    Borsh,
    #[cfg(feature = "deser_json")]
    Json,
}
impl Deser {
    /// Deserialize `T` from the given `[u8]`, based on the variant defined in `Deser`.
    ///
    /// # Errors
    ///
    /// All errors are based on the underlying `Deser` variant and any errors those serialization
    /// traits produce.
    pub fn deserialize<B, T>(self, bytes: B) -> Result<T, Error>
    where
        B: AsRef<[u8]>,
        T: Deserialize,
    {
        match self {
            #[cfg(feature = "deser_borsh")]
            Self::Borsh => {
                Ok(
                    <T as borsh::BorshDeserialize>::try_from_slice(bytes.as_ref())
                        // mapping because it's actually a `std::io::Error`, so ?
                        // would convert the wrong type.
                        .map_err(Error::Borsh)?,
                )
            },
            #[cfg(feature = "deser_json")]
            Self::Json => Ok(serde_json::from_slice(bytes.as_ref())?),
        }
    }
    /// Serialize the `T` to a `Vec<u8>`, based on whatever variant is specified by `Deser`.
    ///
    /// # Errors
    ///
    /// All errors are based on the underlying `Deser` variant and any errors those serialization
    /// traits produce.
    pub fn serialize<T>(self, t: &T) -> Result<Vec<u8>, Error>
    where
        T: Serialize,
    {
        match self {
            #[cfg(feature = "deser_borsh")]
            Self::Borsh => {
                Ok(<T as borsh::BorshSerialize>::try_to_vec(t)
                    // mapping because it's actually a `std::io::Error`, so ?
                    // would convert the wrong type.
                    .map_err(Error::Borsh)?)
            },
            #[cfg(feature = "deser_json")]
            Self::Json => Ok(cjson::to_vec(t)?),
        }
    }
}
#[cfg(all(feature = "deser_borsh"))]
impl Default for Deser {
    fn default() -> Self {
        Self::Borsh
    }
}
#[cfg(all(feature = "deser_json", not(feature = "deser_borsh")))]
impl Default for Deser {
    fn default() -> Self {
        Self::Json
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`borsh`] crate error.
    ///
    /// They return an io::Error, the std::io type is not a bug.
    #[cfg(feature = "deser_borsh")]
    #[error("borsh error: `{0:?}`")]
    Borsh(std::io::Error),
    /// A [`serde_json`] crate error.
    #[cfg(feature = "deser_json")]
    #[error("serde_json error: `{0:?}`")]
    SerdeJson(#[from] serde_json::Error),
    /// A [`cjson`] crate error.
    #[cfg(feature = "deser_json")]
    #[error("cjson error: `{0:?}`")]
    Cjson(cjson::Error),
}
#[cfg(feature = "deser_json")]
impl From<cjson::Error> for Error {
    fn from(err: cjson::Error) -> Self {
        Self::Cjson(err)
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::value::Value};
    #[cfg(feature = "deser_borsh")]
    #[tokio::test]
    async fn borsh() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let expected = Value::from(1);
        let bytes = Deser::serialize(Deser::Borsh, &expected).unwrap();
        let got = Deser::deserialize::<_, Value>(Deser::Borsh, &bytes).unwrap();
        assert_eq!(expected, got);
    }
    #[cfg(feature = "deser_json")]
    #[tokio::test]
    async fn json() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let expected = Value::from(1);
        let bytes = Deser::serialize(Deser::Json, &expected).unwrap();
        let got = Deser::deserialize::<_, Value>(Deser::Json, &bytes).unwrap();
        assert_eq!(expected, got);
    }
}
