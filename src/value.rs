#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Scalar {
    Addr(Addr),
    Uint32(u32),
}
impl From<u32> for Scalar {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
    }
}
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    Addr(Addr),
    Uint32(u32),
    Vec(Vec<Scalar>),
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
