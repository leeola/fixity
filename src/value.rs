mod addr;
pub mod from_cli_str;
mod key;
mod scalar;
use {crate::Error, std::fmt};
pub use {
    addr::Addr,
    key::{Key, KeyOwned},
    scalar::{Scalar, ScalarOwned},
};
pub type ValueOwned = Value<Addr, String, Vec<ScalarOwned>>;
pub type ArchivedValue =
    Value<rkyv::Archived<Addr>, rkyv::Archived<String>, rkyv::Archived<Vec<ScalarOwned>>>;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Value<A, S, V> {
    Addr(A),
    Uint32(u32),
    String(S),
    Vec(V),
}
impl<A, S, V> Value<A, S, V> {
    // I wish we had this :(
    // type Owned = Value<Addr, String, Vec<ScalarOwned>>;
    /// Return the underlying `Addr` if the variant is an `Addr`, `None` otherwise.
    pub fn addr(&self) -> Option<&Addr>
    where
        A: AsRef<Addr>,
    {
        match self {
            Self::Addr(addr) => Some(addr.as_ref()),
            _ => None,
        }
    }
    /// Return the underlying `Addr` if the variant is an `Addr`, `None` otherwise.
    pub fn into_addr(self) -> Option<Addr>
    where
        A: Into<Addr>,
    {
        match self {
            Self::Addr(addr) => Some(addr.into()),
            _ => None,
        }
    }
    fn fmt_variant(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    where
        A: fmt::Debug,
        S: fmt::Debug,
        V: AsRef<[Scalar<A, S>]>,
    {
        use fmt::Debug;
        match self {
            Self::Addr(v) => {
                v.fmt(f)?;
            },
            Self::Uint32(v) => {
                f.write_str("Uint32(")?;
                write!(f, "{})", v)?;
            },
            Self::String(v) => {
                v.fmt(f)?;
            },
            Self::Vec(v) => {
                f.write_str("Vec([\n")?;
                let iter = v.as_ref();
                for elm in iter {
                    f.write_str("    ")?;
                    elm.fmt(f)?;
                    f.write_str(",\n")?;
                }
                f.write_str("])")?;
            },
        }
        Ok(())
    }
}
impl<A, S, V> fmt::Display for Value<A, S, V>
where
    A: fmt::Display,
    S: fmt::Display,
    V: AsRef<[Scalar<A, S>]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addr(v) => write!(f, "{}", v),
            Self::Uint32(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Vec(v) => write!(
                f,
                "{}",
                v.as_ref()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}
impl<A, S, V> fmt::Debug for Value<A, S, V>
where
    A: fmt::Debug,
    S: fmt::Debug,
    V: AsRef<[Scalar<A, S>]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ValueOwned::")?;
        self.fmt_variant(f)
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
impl<T> From<T> for ValueOwned
where
    T: Into<ScalarOwned>,
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
    use super::{Addr, ArchivedValue, ScalarOwned, ValueOwned};
    pub enum ValueOwnedResolver
    where
        Addr: rkyv::Archive,
        u32: rkyv::Archive,
        String: rkyv::Archive,
        Vec<ScalarOwned>: rkyv::Archive,
    {
        Addr(rkyv::Resolver<Addr>),
        Uint32(rkyv::Resolver<u32>),
        String(rkyv::Resolver<String>),
        Vec(rkyv::Resolver<Vec<ScalarOwned>>),
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
            Vec<ScalarOwned>: rkyv::Archive;
        #[repr(C)]
        struct ArchivedVariantUint32(ArchivedTag, rkyv::Archived<u32>, PhantomData<()>)
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<ScalarOwned>: rkyv::Archive;
        #[repr(C)]
        struct ArchivedVariantString(ArchivedTag, rkyv::Archived<String>, PhantomData<()>)
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<ScalarOwned>: rkyv::Archive;
        #[repr(C)]
        struct ArchivedVariantVec(
            ArchivedTag,
            rkyv::Archived<Vec<ScalarOwned>>,
            PhantomData<()>,
        )
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<ScalarOwned>: rkyv::Archive;
        impl Archive for ValueOwned
        where
            Addr: rkyv::Archive,
            u32: rkyv::Archive,
            String: rkyv::Archive,
            Vec<ScalarOwned>: rkyv::Archive,
        {
            type Archived = ArchivedValue;
            type Resolver = ValueOwnedResolver;
            fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                match resolver {
                    ValueOwnedResolver::Addr(resolver_0) => {
                        if let ValueOwned::Addr(self_0) = self {
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
                    ValueOwnedResolver::Uint32(resolver_0) => {
                        if let ValueOwned::Uint32(self_0) = self {
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
                    ValueOwnedResolver::String(resolver_0) => {
                        if let ValueOwned::String(self_0) = self {
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
                    ValueOwnedResolver::Vec(resolver_0) => {
                        if let ValueOwned::Vec(self_0) = self {
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
        impl<__S: Fallible + ?Sized> Serialize<__S> for ValueOwned
        where
            Addr: rkyv::Serialize<__S>,
            u32: rkyv::Serialize<__S>,
            String: rkyv::Serialize<__S>,
            Vec<ScalarOwned>: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(match self {
                    Self::Addr(_0) => {
                        ValueOwnedResolver::Addr(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::Uint32(_0) => {
                        ValueOwnedResolver::Uint32(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::String(_0) => {
                        ValueOwnedResolver::String(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::Vec(_0) => {
                        ValueOwnedResolver::Vec(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                })
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized> Deserialize<ValueOwned, __D> for Archived<ValueOwned>
        where
            Addr: Archive,
            Archived<Addr>: Deserialize<Addr, __D>,
            u32: Archive,
            Archived<u32>: Deserialize<u32, __D>,
            String: Archive,
            Archived<String>: Deserialize<String, __D>,
            Vec<ScalarOwned>: Archive,
            Archived<Vec<ScalarOwned>>: Deserialize<Vec<ScalarOwned>, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<ValueOwned, __D::Error> {
                Ok(match self {
                    Self::Addr(_0) => ValueOwned::Addr(_0.deserialize(deserializer)?),
                    Self::Uint32(_0) => ValueOwned::Uint32(_0.deserialize(deserializer)?),
                    Self::String(_0) => ValueOwned::String(_0.deserialize(deserializer)?),
                    Self::Vec(_0) => ValueOwned::Vec(_0.deserialize(deserializer)?),
                })
            }
        }
    };
    #[cfg(test)]
    use {
        super::{Scalar, Value},
        std::{fmt::Debug, ops::Deref},
    };
    #[cfg(test)]
    fn print_value<A, S, V>(value: &Value<A, S, V>)
    where
        A: Debug + AsRef<Addr>,
        S: Debug + AsRef<str>,
        V: Deref<Target = [Scalar<A, S>]>,
    {
        match value {
            Value::Addr(a) => println!("addr, {}", a.as_ref()),
            Value::Uint32(i) => println!("got int, {}", i),
            Value::String(s) => println!("got string, {:?}", s.as_ref()),
            Value::Vec(v) => {
                let s = v.as_ref();
                println!("got vec, len:{}", s.len());
                for (idx, elm) in s.iter().enumerate() {
                    match elm {
                        Scalar::Addr(a) => println!("i:{}, addr, {}", idx, a.as_ref()),
                        Scalar::Uint32(i) => println!("i:{}, got int, {}", idx, i),
                        Scalar::String(s) => println!("i:{}, got string, {:?}", idx, s.as_ref()),
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
            ValueOwned::String(String::from("foo")),
            ValueOwned::Addr(Addr::hash("foo")),
            ValueOwned::Uint32(42),
            ValueOwned::String(String::from("foo bar baz")),
            ValueOwned::Vec(vec![
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
            let archived = unsafe { archived_value::<ValueOwned>(buf.as_ref(), pos) };
            let deserialized = archived.deserialize(&mut AllocDeserializer).unwrap();
            assert_eq!(owned, deserialized);
            print_value(archived);
            print_value(&owned);
        }
    }
}
