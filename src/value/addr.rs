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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
mod rkyv {
    use super::Addr;
    pub struct AddrResolver(rkyv::Resolver<[u8; 32]>)
    where
        [u8; 32]: rkyv::Archive;
    const _: () = {
        use core::marker::PhantomData;
        use rkyv::{offset_of, Archive};
        impl Archive for Addr
        where
            [u8; 32]: rkyv::Archive,
        {
            type Archived = Addr;
            type Resolver = AddrResolver;
            fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                Addr(self.0.resolve(pos + offset_of!(Addr, 0), resolver.0))
            }
        }
    };
    const _: () = {
        use rkyv::{Fallible, Serialize};
        impl<__S: Fallible + ?Sized> Serialize<__S> for Addr
        where
            [u8; 32]: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(AddrResolver(Serialize::<__S>::serialize(
                    &self.0, serializer,
                )?))
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized> Deserialize<Addr, __D> for Archived<Addr>
        where
            [u8; 32]: Archive,
            Archived<[u8; 32]>: Deserialize<[u8; 32], __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<Addr, __D::Error> {
                Ok(Self(self.0.deserialize(deserializer)?))
            }
        }
    };
    #[cfg(test)]
    fn print_addr(addr: &Addr) {
        println!("addr, {:?}", addr);
    }
    #[test]
    fn rkyv_deser() {
        use rkyv::{
            archived_value,
            de::deserializers::AllocDeserializer,
            ser::{serializers::WriteSerializer, Serializer},
            Deserialize,
        };
        // TODO: use a proptest.
        let values = vec![Addr::hash("foo"), Addr::hash("bar"), Addr::hash("baz")];
        for owned in values {
            let mut serializer = WriteSerializer::new(Vec::new());
            let pos = serializer
                .serialize_value(&owned)
                .expect("failed to serialize value");
            let buf = serializer.into_inner();
            let archived = unsafe { archived_value::<Addr>(buf.as_ref(), pos) };
            let deserialized = archived.deserialize(&mut AllocDeserializer).unwrap();
            assert_eq!(owned, deserialized);
            print_addr(archived);
            print_addr(&owned);
        }
    }
}
