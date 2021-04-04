use {
    super::{Addr, ScalarOwned, Value, ValueOwned},
    std::fmt,
};

pub type KeyOwned = Key<Addr, String, Vec<ScalarOwned>>;
pub type ArchivedKey =
    Key<rkyv::Archived<Addr>, rkyv::Archived<String>, rkyv::Archived<Vec<ScalarOwned>>>;
/// KeyOwned exists as a very thin layer over a [`Value`] for ease of use and reading.
///
/// Ultimately there is no difference between a KeyOwned and a Value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<A, S, V>(pub Value<A, S, V>);
impl fmt::Display for KeyOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
// NIT: maybe make this debug fmt to `KeyOwned::Addr`/etc?
impl fmt::Debug for KeyOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KeyOwned::")?;
        self.0.fmt_variant(f)?;
        f.write_str(")")
    }
}
impl<T> From<T> for KeyOwned
where
    T: Into<ValueOwned>,
{
    fn from(t: T) -> Self {
        Self(t.into())
    }
}
mod rkyv_impl {
    use super::{ArchivedKey, Key, KeyOwned, Value, ValueOwned};
    pub struct KeyOwnedResolver(rkyv::Resolver<ValueOwned>)
    where
        ValueOwned: rkyv::Archive;
    const _: () = {
        use rkyv::{offset_of, Archive};
        impl Archive for KeyOwned
        where
            ValueOwned: rkyv::Archive,
        {
            type Archived = ArchivedKey;
            type Resolver = KeyOwnedResolver;
            fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                Key(self.0.resolve(pos + offset_of!(ArchivedKey, 0), resolver.0))
            }
        }
    };
    const _: () = {
        use rkyv::{Fallible, Serialize};
        impl<__S: Fallible + ?Sized> Serialize<__S> for KeyOwned
        where
            ValueOwned: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(KeyOwnedResolver(Serialize::<__S>::serialize(
                    &self.0, serializer,
                )?))
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized> Deserialize<KeyOwned, __D> for Archived<KeyOwned>
        where
            ValueOwned: Archive,
            Archived<ValueOwned>: Deserialize<ValueOwned, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<KeyOwned, __D::Error> {
                Ok(Key(self.0.deserialize(deserializer)?))
            }
        }
    };
    #[cfg(test)]
    use {
        super::super::{Addr, Scalar, ScalarOwned},
        std::{fmt::Debug, ops::Deref},
    };
    #[cfg(test)]
    fn print_key<A, S, V>(key: &Key<A, S, V>)
    where
        A: Debug + AsRef<Addr>,
        S: Debug + AsRef<str>,
        V: Deref<Target = [Scalar<A, S>]>,
    {
        match &key.0 {
            Value::Addr(a) => println!("addr, {}", a.as_ref()),
            Value::Uint32(i) => println!("got int, {}", i),
            Value::String(s) => println!("got string, {:?}", s.as_ref()),
            Value::Vec(v) => {
                let s = v.deref();
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
        let keys = vec![
            Key(Value::String(String::from("foo"))),
            Key(Value::Addr(Addr::hash("foo"))),
            Key(Value::Uint32(42)),
            Key(Value::String(String::from("foo bar baz"))),
            Key(Value::Vec(vec![
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
            let archived = unsafe { archived_value::<KeyOwned>(buf.as_ref(), pos) };
            let deserialized = archived.deserialize(&mut AllocDeserializer).unwrap();
            assert_eq!(owned, deserialized);
            print_key(archived);
            print_key(&owned);
        }
    }
}
#[cfg(feature = "borsh")]
mod borsh_impl {
    use super::Key;
    impl<A, S, V> borsh::ser::BorshSerialize for Key<A, S, V>
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
    impl<A, S, V> borsh::de::BorshDeserialize for Key<A, S, V>
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
