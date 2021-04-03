use super::Addr;
use std::fmt;

pub type Scalar = ScalarRef<Addr, String>;

pub type ArchivedScalar = ScalarRef<rkyv::Archived<Addr>, rkyv::Archived<String>>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScalarRef<A, S> {
    Addr(A),
    Uint32(u32),
    String(S),
}
impl<A, S> From<u32> for ScalarRef<A, S> {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
    }
}
impl<A> From<&str> for ScalarRef<A, String> {
    fn from(t: &str) -> Self {
        Self::String(t.to_owned())
    }
}
impl<A> From<String> for ScalarRef<A, String> {
    fn from(t: String) -> Self {
        Self::String(t)
    }
}
impl<S> From<Addr> for ScalarRef<Addr, S> {
    fn from(t: Addr) -> Self {
        Self::Addr(t)
    }
}
impl<A, S> fmt::Display for ScalarRef<A, S>
where
    A: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addr(v) => write!(f, "{}", v),
            Self::Uint32(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
        }
    }
}
mod derived_rkyv {
    use super::*;
    pub enum ScalarResolver
    where
        Addr: rkyv::Archive,
        u32: rkyv::Archive,
        String: rkyv::Archive,
    {
        Addr(rkyv::Resolver<Addr>),
        Uint32(rkyv::Resolver<u32>),
        String(rkyv::Resolver<String>),
    }
    const _: () =
        {
            use core::marker::PhantomData;
            use rkyv::{offset_of, Archive};
            #[repr(u8)]
            enum ArchivedTag {
                Addr,
                Uint32,
                String,
            }
            #[repr(C)]
            struct ArchivedVariantAddr(ArchivedTag, rkyv::Archived<Addr>, PhantomData<()>)
            where
                Addr: rkyv::Archive,
                u32: rkyv::Archive,
                String: rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantUint32(ArchivedTag, rkyv::Archived<u32>, PhantomData<()>)
            where
                Addr: rkyv::Archive,
                u32: rkyv::Archive,
                String: rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantString(ArchivedTag, rkyv::Archived<String>, PhantomData<()>)
            where
                Addr: rkyv::Archive,
                u32: rkyv::Archive,
                String: rkyv::Archive;
            impl Archive for Scalar
            where
                Addr: rkyv::Archive,
                u32: rkyv::Archive,
                String: rkyv::Archive,
            {
                type Archived = ArchivedScalar;
                type Resolver = ScalarResolver;
                fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                    match resolver {
                        ScalarResolver::Addr(resolver_0) => {
                            if let Scalar::Addr(self_0) = self {
                                ArchivedScalar::Addr(
                                    self_0.resolve(
                                        pos + offset_of!(ArchivedVariantAddr, 1),
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
                        ScalarResolver::Uint32(resolver_0) => {
                            if let Scalar::Uint32(self_0) = self {
                                ArchivedScalar::Uint32(self_0.resolve(
                                    pos + offset_of!(ArchivedVariantUint32, 1),
                                    resolver_0,
                                ))
                            } else {
                                {
                                    // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                    //::std::rt::begin_panic(
                                    panic!("enum resolver variant does not match value variant",)
                                }
                            }
                        },
                        ScalarResolver::String(resolver_0) => {
                            if let Scalar::String(self_0) = self {
                                ArchivedScalar::String(self_0.resolve(
                                    pos + offset_of!(ArchivedVariantString, 1),
                                    resolver_0,
                                ))
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
        impl<__S: Fallible + ?Sized> Serialize<__S> for Scalar
        where
            Addr: rkyv::Serialize<__S>,
            u32: rkyv::Serialize<__S>,
            String: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(match self {
                    Self::Addr(_0) => {
                        ScalarResolver::Addr(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::Uint32(_0) => {
                        ScalarResolver::Uint32(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::String(_0) => {
                        ScalarResolver::String(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                })
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized> Deserialize<Scalar, __D> for Archived<Scalar>
        where
            Addr: Archive,
            Archived<Addr>: Deserialize<Addr, __D>,
            u32: Archive,
            Archived<u32>: Deserialize<u32, __D>,
            String: Archive,
            Archived<String>: Deserialize<String, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<Scalar, __D::Error> {
                Ok(match self {
                    Self::Addr(_0) => Scalar::Addr(_0.deserialize(deserializer)?),
                    Self::Uint32(_0) => Scalar::Uint32(_0.deserialize(deserializer)?),
                    Self::String(_0) => Scalar::String(_0.deserialize(deserializer)?),
                })
            }
        }
    };
    #[cfg(test)]
    fn print_scalar<A, S>(scalar: &ScalarRef<A, S>)
    where
        A: AsRef<Addr>,
        S: AsRef<str>,
    {
        match scalar {
            ScalarRef::Addr(a) => println!("addr, {}", a.as_ref()),
            ScalarRef::Uint32(i) => println!("got int, {}", i),
            ScalarRef::String(s) => println!("got string, {:?}", s.as_ref()),
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
            Scalar::String(String::from("foo")),
            Scalar::Addr(Addr::hash("foo")),
            Scalar::Uint32(42),
            Scalar::String(String::from("foo bar baz")),
        ];
        for owned in values {
            let mut serializer = WriteSerializer::new(Vec::new());
            let pos = serializer
                .serialize_value(&owned)
                .expect("failed to serialize value");
            let buf = serializer.into_inner();
            let archived = unsafe { archived_value::<Scalar>(buf.as_ref(), pos) };
            let deserialized = archived.deserialize(&mut AllocDeserializer).unwrap();
            assert_eq!(owned, deserialized);
            print_scalar(archived);
            print_scalar(&owned);
        }
    }
}
