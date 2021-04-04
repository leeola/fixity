use {
    super::{Addr, Scalar, Value, ValueRef},
    std::fmt,
};

pub type Key = KeyRef<Addr, String, Vec<Scalar>>;
pub type ArchivedKey =
    KeyRef<rkyv::Archived<Addr>, rkyv::Archived<String>, rkyv::Archived<Vec<Scalar>>>;
/// Key exists as a very thin layer over a [`Value`] for ease of use and reading.
///
/// Ultimately there is no difference between a Key and a Value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyRef<A, S, V>(pub ValueRef<A, S, V>);
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
mod rkyv_impl {
    use super::{ArchivedKey, Key, KeyRef, Value};
    pub struct KeyResolver(rkyv::Resolver<Value>)
    where
        Value: rkyv::Archive;
    const _: () = {
        use rkyv::{offset_of, Archive};
        impl Archive for Key
        where
            Value: rkyv::Archive,
        {
            type Archived = ArchivedKey;
            type Resolver = KeyResolver;
            fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                KeyRef(self.0.resolve(
                    pos + offset_of!(ArchivedKey, 0),
                    //pos + {
                    //    let uninit =
                    // ::memoffset::__priv::mem::MaybeUninit::<ArchivedKey>::uninit();
                    //    let base_ptr: *const ArchivedKey = uninit.as_ptr();
                    //    let field_ptr = {
                    //        #[allow(clippy::unneeded_field_pattern)]
                    //        let ArchivedKey { 0: _, .. };
                    //        let base = base_ptr;
                    //        #[allow(unused_unsafe)]
                    //        unsafe {
                    //            {
                    //                &raw const (*(base as *const ArchivedKey)).0
                    //            }
                    //        }
                    //    };
                    //    (field_ptr as usize) - (base_ptr as usize)
                    //},
                    resolver.0,
                ))
            }
        }
    };
    const _: () = {
        use rkyv::{Fallible, Serialize};
        impl<__S: Fallible + ?Sized> Serialize<__S> for Key
        where
            Value: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(KeyResolver(Serialize::<__S>::serialize(
                    &self.0, serializer,
                )?))
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized> Deserialize<Key, __D> for Archived<Key>
        where
            Value: Archive,
            Archived<Value>: Deserialize<Value, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<Key, __D::Error> {
                Ok(KeyRef(self.0.deserialize(deserializer)?))
            }
        }
    };
    #[cfg(test)]
    use {
        super::super::{Addr, Scalar, ScalarRef, ValueRef},
        std::{fmt::Debug, ops::Deref},
    };
    #[cfg(test)]
    fn print_key<A, S, V>(key: &KeyRef<A, S, V>)
    where
        A: Debug + AsRef<Addr>,
        S: Debug + AsRef<str>,
        V: Deref<Target = [ScalarRef<A, S>]>,
    {
        match &key.0 {
            ValueRef::Addr(a) => println!("addr, {}", a.as_ref()),
            ValueRef::Uint32(i) => println!("got int, {}", i),
            ValueRef::String(s) => println!("got string, {:?}", s.as_ref()),
            ValueRef::Vec(v) => {
                let s = v.deref();
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
        let keys = vec![
            KeyRef(Value::String(String::from("foo"))),
            KeyRef(Value::Addr(Addr::hash("foo"))),
            KeyRef(Value::Uint32(42)),
            KeyRef(Value::String(String::from("foo bar baz"))),
            KeyRef(Value::Vec(vec![
                Scalar::String(String::from("foo")),
                Scalar::Addr(Addr::hash("foo")),
                Scalar::Uint32(42),
                Scalar::String(String::from("foo bar baz")),
            ])),
        ];
        for owned in keys {
            let mut serializer = WriteSerializer::new(Vec::new());
            let pos = serializer
                .serialize_value(&owned)
                .expect("failed to serialize value");
            let buf = serializer.into_inner();
            let archived = unsafe { archived_value::<Key>(buf.as_ref(), pos) };
            let deserialized = archived.deserialize(&mut AllocDeserializer).unwrap();
            assert_eq!(owned, deserialized);
            print_key(archived);
            print_key(&owned);
        }
    }
}
#[cfg(feature = "borsh")]
mod borsh_impl {
    use super::KeyRef;
    impl<A, S, V> borsh::ser::BorshSerialize for KeyRef<A, S, V>
    where
        A: borsh::ser::BorshSerialize,
        S: borsh::ser::BorshSerialize,
        V: borsh::ser::BorshSerialize,
    {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
            borsh::BorshSerialize::serialize(&self.0, writer)?;
            Ok(())
        }
    }
    impl<A, S, V> borsh::de::BorshDeserialize for KeyRef<A, S, V>
    where
        A: borsh::de::BorshDeserialize,
        S: borsh::de::BorshDeserialize,
        V: borsh::de::BorshDeserialize,
    {
        fn deserialize(
            buf: &mut &[u8],
        ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
            Ok(Self(borsh::BorshDeserialize::deserialize(buf)?))
        }
    }
}
