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
    addr::Addr,
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
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
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
mod rkyv_impl {
    use super::{Addr, Scalar};
    pub type Value = ValueRef<Addr, String, Vec<Scalar>>;
    pub type ArchivedValue =
        ValueRef<rkyv::Archived<Addr>, rkyv::Archived<String>, rkyv::Archived<Vec<Scalar>>>;
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(u8)]
    pub enum ValueRef<A, S, V> {
        Addr(A),
        Uint32(u32),
        String(S),
        Vec(V),
    }
    pub enum ValueResolver
    where
        Addr: rkyv::Archive,
        u32: rkyv::Archive,
        String: rkyv::Archive,
        Vec<Scalar>: rkyv::Archive,
    {
        Addr(rkyv::Resolver<Addr>),
        Uint32(rkyv::Resolver<u32>),
        String(rkyv::Resolver<String>),
        Vec(rkyv::Resolver<Vec<Scalar>>),
    }
    const _: () = {
        use core::marker::PhantomData;
        use rkyv::{offset_of, Archive};
        #[repr(u8)]
        enum ArchivedTag {
            Addr,
            Uint32,
            String,
            Vec,
        }
        #[repr(C)]
        struct ArchivedVariantAddr(ArchivedTag, rkyv::Archived<Addr>, PhantomData<()>)
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<Scalar>: rkyv::Archive;
        #[repr(C)]
        struct ArchivedVariantUint32(ArchivedTag, rkyv::Archived<u32>, PhantomData<()>)
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<Scalar>: rkyv::Archive;
        #[repr(C)]
        struct ArchivedVariantString(ArchivedTag, rkyv::Archived<String>, PhantomData<()>)
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<Scalar>: rkyv::Archive;
        #[repr(C)]
        struct ArchivedVariantVec(ArchivedTag, rkyv::Archived<Vec<Scalar>>, PhantomData<()>)
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<Scalar>: rkyv::Archive;
        impl Archive for Value
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<Scalar>: rkyv::Archive,
        {
            type Archived = ArchivedValue;
            type Resolver = ValueResolver;
            fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                match resolver {
                    ValueResolver::Addr(resolver_0) => {
                        if let Value::Addr(self_0) = self {
                            ArchivedValue::Addr(
                                self_0
                                    .resolve(pos + offset_of!(ArchivedVariantAddr, 1), resolver_0),
                            )
                        } else {
                            {
                                // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                //::std::rt::begin_panic(
                                panic!("enum resolver variant does not match value variant",)
                            }
                        }
                    },
                    ValueResolver::Uint32(resolver_0) => {
                        if let Value::Uint32(self_0) = self {
                            ArchivedValue::Uint32(
                                self_0.resolve(
                                    pos + offset_of!(ArchivedVariantUint32, 1),
                                    resolver_0,
                                ),
                            )
                        } else {
                            {
                                // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                //::std::rt::begin_panic(
                                panic!("enum resolver variant does not match value variant",)
                            }
                        }
                    },
                    ValueResolver::String(resolver_0) => {
                        if let Value::String(self_0) = self {
                            ArchivedValue::String(
                                self_0.resolve(
                                    pos + offset_of!(ArchivedVariantString, 1),
                                    resolver_0,
                                ),
                            )
                        } else {
                            {
                                // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                //::std::rt::begin_panic(
                                panic!("enum resolver variant does not match value variant",)
                            }
                        }
                    },
                    ValueResolver::Vec(resolver_0) => {
                        if let Value::Vec(self_0) = self {
                            ArchivedValue::Vec(
                                self_0.resolve(pos + offset_of!(ArchivedVariantVec, 1), resolver_0),
                            )
                        } else {
                            {
                                // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                //::std::rt::begin_panic(
                                panic!("enum resolver variant does not match value variant",)
                            }
                        }
                    },
                }
            }
        }
    };
    const _: () = {
        use rkyv::{Fallible, Serialize};
        impl<__S: Fallible + ?Sized> Serialize<__S> for Value
        where
            Addr: rkyv::Serialize<__S>,
            u32: rkyv::Serialize<__S>,
            String: rkyv::Serialize<__S>,
            Vec<Scalar>: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(match self {
                    Self::Addr(_0) => {
                        ValueResolver::Addr(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::Uint32(_0) => {
                        ValueResolver::Uint32(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::String(_0) => {
                        ValueResolver::String(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::Vec(_0) => {
                        ValueResolver::Vec(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                })
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized> Deserialize<Value, __D> for Archived<Value>
        where
            Addr: Archive,
            Archived<Addr>: Deserialize<Addr, __D>,
            u32: Archive,
            Archived<u32>: Deserialize<u32, __D>,
            String: Archive,
            Archived<String>: Deserialize<String, __D>,
            Vec<Scalar>: Archive,
            Archived<Vec<Scalar>>: Deserialize<Vec<Scalar>, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<Value, __D::Error> {
                Ok(match self {
                    Self::Addr(_0) => Value::Addr(_0.deserialize(deserializer)?),
                    Self::Uint32(_0) => Value::Uint32(_0.deserialize(deserializer)?),
                    Self::String(_0) => Value::String(_0.deserialize(deserializer)?),
                    Self::Vec(_0) => Value::Vec(_0.deserialize(deserializer)?),
                })
            }
        }
    };
    #[cfg(test)]
    use {
        super::ScalarRef,
        std::{fmt::Debug, ops::Deref},
    };
    #[cfg(test)]
    fn print_value<A, S, V>(value: &ValueRef<A, S, V>)
    where
        A: Debug + AsRef<Addr>,
        S: Debug + AsRef<str>,
        V: Deref<Target = [ScalarRef<A, S>]>,
    {
        match value {
            ValueRef::Addr(a) => println!("addr, {}", a.as_ref()),
            ValueRef::Uint32(i) => println!("got int, {}", i),
            ValueRef::String(s) => println!("got string, {:?}", s.as_ref()),
            ValueRef::Vec(v) => {
                let s = v.as_ref();
                println!("got vec, len:{}", s.len());
                for (idx, elm) in s.iter().enumerate() {
                    match elm {
                        ScalarRef::Addr(a) => println!("i:{}, addr, {}", idx, a.as_ref()),
                        ScalarRef::Uint32(i) => println!("i:{}, got int, {}", idx, i),
                        ScalarRef::String(s) => println!("i:{}, got string, {:?}", idx, s.as_ref()),
                    }
                }
            },
        }
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
        let values = vec![
            Value::String(String::from("foo")),
            Value::Addr(Addr::hash("foo")),
            Value::Uint32(42),
            Value::String(String::from("foo bar baz")),
            Value::Vec(vec![
                Scalar::String(String::from("foo")),
                Scalar::Addr(Addr::hash("foo")),
                Scalar::Uint32(42),
                Scalar::String(String::from("foo bar baz")),
            ]),
        ];
        for owned in values {
            let mut serializer = WriteSerializer::new(Vec::new());
            let pos = serializer
                .serialize_value(&owned)
                .expect("failed to serialize value");
            let buf = serializer.into_inner();
            let archived = unsafe { archived_value::<Value>(buf.as_ref(), pos) };
            let deserialized = archived.deserialize(&mut AllocDeserializer).unwrap();
            assert_eq!(owned, deserialized);
            print_value(archived);
            print_value(&owned);
        }
    }
}
