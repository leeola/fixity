use super::{Addr, ArchivedAddr};
use std::fmt;

pub mod zomg {
    use super::Addr;

    #[derive(
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    pub enum Scalar {
        Addr(Addr),
        Uint32(u32),
        String(String),
    }
}

pub type Scalar = ScalarRef<Addr, u32, String>;

pub type ArchivedScalar =
    ScalarRef<rkyv::Archived<Addr>, rkyv::Archived<u32>, rkyv::Archived<String>>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScalarRef<A, U32, S> {
    Addr(A),
    Uint32(U32),
    String(S),
}
impl<A, S> From<u32> for ScalarRef<A, u32, S> {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
    }
}
impl<A, U32> From<&str> for ScalarRef<A, U32, String> {
    fn from(t: &str) -> Self {
        Self::String(t.to_owned())
    }
}
impl<U32, S> From<Addr> for ScalarRef<Addr, U32, S> {
    fn from(t: Addr) -> Self {
        Self::Addr(t)
    }
}
impl<A, U32, S> fmt::Display for ScalarRef<A, U32, S>
where
    A: fmt::Display,
    U32: fmt::Display,
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
        use rkyv::{Archive, Fallible, Serialize};
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
}

#[cfg(test)]
fn print_scalar<A, U32, S>(scalar: &ScalarRef<A, U32, S>)
where
    U32: Copy + Into<u32>,
    S: AsRef<str>,
{
    match scalar {
        ScalarRef::Addr(_) => unimplemented!(),
        &ScalarRef::Uint32(i) => println!("got int, {}", i.into()),
        ScalarRef::String(s) => println!("got string, {:?}", s.as_ref()),
    }
}
#[test]
fn rkyv_deser() {
    use {
        rkyv::{
            archived_value,
            de::deserializers::AllocDeserializer,
            ser::{serializers::WriteSerializer, Serializer},
            Deserialize,
        },
    };
    let owned = Scalar::String(String::from("foo"));
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
