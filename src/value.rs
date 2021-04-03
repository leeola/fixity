mod addr;
pub mod from_cli_str;
mod scalar;
use {
    crate::Error,
    std::{
        convert::{TryFrom, TryInto},
        fmt,
    },
};
pub use {
    addr::{Addr, ArchivedAddr},
    scalar::{Scalar, ScalarRef},
};
/// Key exists as a very thin layer over a [`Value`] for ease of use and reading.
///
/// Ultimately there is no difference between a Key and a Value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(pub Value);
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
// NIT: maybe make this debug fmt to `Key::Addr`/etc?
impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key::")?;
        self.0.fmt_variant(f)?;
        f.write_str(")")
    }
}
impl<T> From<T> for Key
where
    T: Into<Value>,
{
    fn from(t: T) -> Self {
        Self(t.into())
    }
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Addr(Addr),
    Uint32(u32),
    String(String),
    Vec(Vec<Scalar>),
}
impl Value {
    /// Return the underlying `Addr` if the variant is an `Addr`, `None` otherwise.
    pub fn addr(&self) -> Option<&Addr> {
        match self {
            Self::Addr(addr) => Some(addr),
            _ => None,
        }
    }
    /// Return the underlying `Addr` if the variant is an `Addr`, `None` otherwise.
    pub fn into_addr(self) -> Option<Addr> {
        match self {
            Self::Addr(addr) => Some(addr),
            _ => None,
        }
    }
    fn fmt_variant(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Debug;
        match self {
            Self::Addr(v) => {
                f.write_str("Addr(")?;
                // TODO: is there a way we can encode this without allocating? Perhaps into
                // a different encoding?
                f.write_str(v.long().as_str())?;
            },
            Self::Uint32(v) => {
                f.write_str("Uint32(")?;
                write!(f, "{}", v)?;
            },
            Self::String(v) => {
                f.write_str("String(")?;
                f.write_str(v.as_str())?;
            },
            Self::Vec(v) => {
                f.write_str("Vec([\n")?;
                let iter = v.iter();
                for elm in iter {
                    f.write_str("    ")?;
                    elm.fmt(f)?;
                    f.write_str(",\n")?;
                }
                f.write_str("]")?;
            },
        }
        Ok(())
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addr(v) => write!(f, "{}", v),
            Self::Uint32(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Vec(v) => write!(
                f,
                "{}",
                v.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Value::")?;
        self.fmt_variant(f)?;
        f.write_str(")")
    }
}
/// A helper to centralize serialization logic for a potential future
/// where we change/tweak/configure serialization.
///
/// How we handle schema/serialization compatibility is TBD.
#[cfg(not(feature = "borsh"))]
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
    t.try_to_vec()
        // mapping because it's actually a `std::io::Error`, so ?
        // would convert the wrong type.
        .map_err(Error::Borsh)
}
/// A helper to centralize deserialization logic for a potential future
/// where we change/tweak/configure deserialization.
///
/// How we handle schema/deserialization compatibility is TBD.
#[cfg(not(feature = "borsh"))]
pub fn deserialize<T>(_: T) -> Result<Vec<u8>, Error> {
    Err(Error::Unhandled("deserializer not configured".into()))
}
#[cfg(feature = "borsh")]
/// A helper to centralize deserialization logic for a potential future
/// where we change/tweak/configure deserialization.
///
/// How we handle schema/deserialization compatibility is TBD.
pub fn deserialize<T>(bytes: &[u8]) -> Result<T, Error>
where
    T: borsh::BorshDeserialize,
{
    T::try_from_slice(bytes)
        // mapping because it's actually a `std::io::Error`, so ?
        // would convert the wrong type.
        .map_err(Error::Borsh)
}
/// A helper to centralize deserialization logic for a potential future
/// where we change/tweak/configure deserialization.
///
/// How we handle schema/deserialization compatibility is TBD.
#[cfg(not(feature = "borsh"))]
pub fn deserialize_with_addr<T>(_: T, _: &Addr) -> Result<Vec<u8>, Error> {
    Err(Error::Unhandled("deserializer not configured".into()))
}
#[cfg(feature = "borsh")]
/// A helper to centralize deserialization logic for a potential future
/// where we change/tweak/configure deserialization.
///
/// How we handle schema/deserialization compatibility is TBD.
pub fn deserialize_with_addr<T>(bytes: &[u8], addr: &Addr) -> Result<T, Error>
where
    T: borsh::BorshDeserialize,
{
    T::try_from_slice(bytes).map_err(|err| Error::BorshAddr {
        addr: addr.clone(),
        err,
    })
}
impl<T> From<T> for Value
where
    T: Into<Scalar>,
{
    fn from(t: T) -> Self {
        match t.into() {
            Scalar::Addr(v) => Self::Addr(v),
            Scalar::Uint32(v) => Self::Uint32(v),
            Scalar::String(v) => Self::String(v),
        }
    }
}
