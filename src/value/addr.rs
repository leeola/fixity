use {
    crate::Error,
    multibase::Base,
    std::{
        convert::{TryFrom, TryInto},
        fmt,
    },
};
const PRIMARY_ENCODING: Base = Base::Base58Btc;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct Addr([u8; 32]);
impl Addr {
    /// The length in bytes of an [`Addr`].
    pub const LEN: usize = 32;
    /// Hash the provided bytes and create an `Addr` of the bytes.
    pub fn hash<B: AsRef<[u8]>>(bytes: B) -> Self {
        let h: [u8; 32] = <[u8; 32]>::from(blake3::hash(bytes.as_ref()));
        Self(h)
    }
    /// Create an `Addr` from a string of encoded bytes.
    ///
    /// If the decoded bytes length does not match `Addr::LEN`, `None` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use fixity::Addr;
    /// let addr1 = Addr::hash("foo");
    /// let addr2 = Addr::decode(addr1.long());
    /// assert_eq!(Some(addr1), addr2);
    /// ```
    ///
    /// Corrupt encodings return None.
    ///
    /// ```rust
    /// # use fixity::Addr;
    /// let addr = Addr::decode("foo");
    /// assert_eq!(addr, None);
    /// ```
    ///
    /// Valid encodings but invalid byte lengths return None.
    ///
    /// ```rust
    /// # use fixity::Addr;
    /// let encoded = multibase::encode(multibase::Base::Base58Btc, &[1,2,3,4]);
    /// let addr = Addr::decode(encoded);
    /// assert_eq!(addr, None);
    /// ```
    pub fn decode<S: AsRef<str>>(s: S) -> Option<Self> {
        let (_, bytes) = multibase::decode(s).ok()?;
        let arr: [u8; 32] = bytes.try_into().ok()?;
        Some(Self(arr))
    }
    /// Return a `Base58Btc` encoded `Addr`, in full.
    pub fn long(&self) -> String {
        multibase::encode(PRIMARY_ENCODING, &self.0)
    }
    /// Convert the underlying String into a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}
impl AsRef<Addr> for Addr {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl From<&Addr> for Addr {
    fn from(t: &Addr) -> Self {
        t.clone()
    }
}
impl TryFrom<Vec<u8>> for Addr {
    type Error = Vec<u8>;
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let arr: [u8; 32] = bytes.try_into()?;
        Ok(Self(arr))
    }
}
impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Addr(")?;
        // TODO: is there a way we can encode this without allocating? Perhaps into
        // a different encoding?
        f.write_str(self.long().as_str())?;
        f.write_str(")")
    }
}
impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: is there a way we can encode this without allocating? Perhaps into
        // a different encoding?
        write!(f, "{}", self.long())
    }
}
