pub mod from_cli_str;

use {
    crate::Error,
    multibase::Base,
    std::{convert::TryFrom, fmt},
};

const ADDR_SHORT_LEN: usize = 8;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct Addr(String);
impl Addr {
    /// The length in bytes of an [`Addr`].
    pub const LEN: usize = 32;
    /// Hash the provided bytes and create an `Addr` of the bytes.
    pub fn from_unhashed_bytes(bytes: &[u8]) -> Self {
        let h = <[u8; 32]>::from(blake3::hash(bytes));
        Self(multibase::encode(Base::Base58Btc, &h))
    }
    /// Create an `Addr` from a `Base::Base58Btc` encoded byte vec.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use fixity::Addr;
    /// let addr1 = Addr::from_unhashed_bytes("foo".as_bytes());
    /// let addr2 = Addr::from_encoded(addr1.clone().long().into_bytes());
    /// assert_eq!(Some(addr1), addr2);
    /// ```
    pub fn from_encoded(bytes: Vec<u8>) -> Option<Self> {
        let s = String::from_utf8(bytes).ok()?;
        Some(Self(s))
    }
    /// Return a partial address which is *usually* unique enough to reference
    /// a content address.
    ///
    /// Useful for a decent UX.
    pub fn short(mut self) -> String {
        let _ = self.0.split_off(ADDR_SHORT_LEN);
        self.0
    }
    /// Return a `Base58Btc` encoded `Addr`, in full.
    pub fn long(self) -> String {
        self.0
    }
    /// Convert the underlying String into a str.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    /// Convert the underlying String into a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
impl TryFrom<Vec<u8>> for Addr {
    type Error = Addr;
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // This will be better in the nearish future, when Addr is converted from
        // an Addr(String) and into an Addr([u8; 32]).
        //
        // For now though, we have to hash it to ensure safe utf8 encoding.
        if bytes.len() != Self::LEN {
            return Err(Self::from_unhashed_bytes(&bytes));
        }
        Ok(Self::from_unhashed_bytes(&bytes))
    }
}
impl std::borrow::Borrow<str> for Addr {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}
impl std::borrow::Borrow<String> for Addr {
    fn borrow(&self) -> &String {
        &self.0
    }
}
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
impl From<&Vec<u8>> for Addr {
    fn from(bytes: &Vec<u8>) -> Self {
        log::warn!("Deprecated From<Bytes> usage");
        Self::from_unhashed_bytes(bytes)
    }
}
impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Addr(")?;
        f.write_str(self.0.as_str())?;
        f.write_str(")")
    }
}
impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: encode the first X bytes, once this becomes a fixed [u8; 32].
        write!(f, "{}", self.clone().short())
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
    String(String),
}
impl From<u32> for Scalar {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
    }
}
impl From<&str> for Scalar {
    fn from(t: &str) -> Self {
        Self::String(t.to_owned())
    }
}
impl From<Addr> for Scalar {
    fn from(t: Addr) -> Self {
        Self::Addr(t)
    }
}
impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addr(v) => write!(f, "{}", v),
            Self::Uint32(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
        }
    }
}
/// Key exists as a very thin layer over a [`Value`] for ease of use and reading.
///
/// Ultimately there is no difference between a Key and a Value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum Value {
    Addr(Addr),
    Uint32(u32),
    String(String),
    Vec(Vec<Scalar>),
}
impl Value {
    fn fmt_variant(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Debug;
        match self {
            Self::Addr(v) => {
                f.write_str("Addr(")?;
                f.write_str(v.as_str())?;
            }
            Self::Uint32(v) => {
                f.write_str("Uint32(")?;
                write!(f, "{}", v)?;
            }
            Self::String(v) => {
                f.write_str("String(")?;
                f.write_str(v.as_str())?;
            }
            Self::Vec(v) => {
                f.write_str("Vec([\n")?;
                let iter = v.iter();
                for elm in iter {
                    f.write_str("    ")?;
                    elm.fmt(f)?;
                    f.write_str(",\n")?;
                }
                f.write_str("]")?;
            }
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
