use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct Addr(String);
impl AsRef<str> for Addr {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
impl From<String> for Addr {
    fn from(hash: String) -> Self {
        Self(hash)
    }
}
impl From<&str> for Addr {
    fn from(hash: &str) -> Self {
        hash.to_owned().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum Scalar {
    Addr(Addr),
    Uint32(u32),
}
impl From<u32> for Scalar {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum Value {
    Addr(Addr),
    Uint32(u32),
    Vec(Vec<Scalar>),
}
#[cfg(not(feature = "borsh"))]
/// A helper to centralize serialization logic for a potential future
/// where we change/tweak/configure serialization.
///
/// How we handle schema/serialization compatibility is TBD.
pub fn serialize<T>(_: T) -> Result<Vec<u8>, Error> {
    Err(Error::Unhandled("serializer not configured".into()))
}
#[cfg(feature = "borsh")]
/// A helper to centralize serialization logic for a potential future
/// where we change/tweak/configure serialization.
///
/// How we handle schema/serialization compatibility is TBD.
pub fn serialize<T>(t: T) -> Result<Vec<u8>, Error>
where
    T: borsh::BorshSerialize,
{
    Ok(t.try_to_vec()
        // mapping because it's actually a `std::io::Error`, so ?
        // would convert the wrong type.
        .map_err(|err| Error::Borsh(err))?)
}
impl<T> From<T> for Value
where
    T: Into<Scalar>,
{
    fn from(t: T) -> Self {
        match t.into() {
            Scalar::Addr(v) => Self::Addr(v),
            Scalar::Uint32(v) => Self::Uint32(v),
        }
    }
}
