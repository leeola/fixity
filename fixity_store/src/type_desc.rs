use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

pub trait TypeDescription {
    fn type_desc() -> ValueDesc;
}
// TODO: Prob worth revisiting the usage of TypeId. Notably it doesn't work with non'static (i
// think[1][2]) lifetimes, so it's prob worth finding an alternate way to represent this information
// here. Importantly though all we care about is a loose ability to describe and compare types,
// notably anything that's [de]serialized, to check for compatibility.
//
// [1]: https://internals.rust-lang.org/t/would-non-static-typeid-be-at-all-possible/14258/6
// [2]: https://docs.rs/better_any/latest/better_any
//
// TODO: a more concise Debug impl.
#[derive(Debug)]
pub enum ValueDesc {
    Number(TypeId),
    String(TypeId),
    Ptr(Box<ValueDesc>),
    Tuple {
        type_id: TypeId,
        values: Vec<ValueDesc>,
    },
    Vec {
        type_id: TypeId,
        value: Box<ValueDesc>,
    },
    Array {
        type_id: TypeId,
        len: usize,
        value: Box<ValueDesc>,
    },
    Struct {
        name: &'static str,
        type_id: TypeId,
        values: Vec<ValueDesc>,
    },
}
impl ValueDesc {
    pub fn of<T: TypeDescription>() -> ValueDesc {
        T::type_desc()
    }
    /// An equality check for the inner type values of a given type.
    pub fn type_eq(&self, _other: &Self) -> bool {
        todo!()
    }
}
// NIT: might need a prettier impl than Debug for Display. Depends on what
// the Debug display ends up being.
impl Display for ValueDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl TypeDescription for u8 {
    fn type_desc() -> ValueDesc {
        ValueDesc::Number(TypeId::of::<Self>())
    }
}
impl TypeDescription for u32 {
    fn type_desc() -> ValueDesc {
        ValueDesc::Number(TypeId::of::<Self>())
    }
}
impl TypeDescription for i32 {
    fn type_desc() -> ValueDesc {
        ValueDesc::Number(TypeId::of::<Self>())
    }
}
impl TypeDescription for u64 {
    fn type_desc() -> ValueDesc {
        ValueDesc::Number(TypeId::of::<Self>())
    }
}
impl TypeDescription for i64 {
    fn type_desc() -> ValueDesc {
        ValueDesc::Number(TypeId::of::<Self>())
    }
}
impl TypeDescription for String {
    fn type_desc() -> ValueDesc {
        ValueDesc::String(TypeId::of::<Self>())
    }
}
impl<const N: usize> TypeDescription for [u8; N] {
    fn type_desc() -> ValueDesc {
        ValueDesc::Array {
            type_id: TypeId::of::<Self>(),
            len: N,
            value: Box::new(ValueDesc::of::<u8>()),
        }
    }
}
// TODO: Make Generic over tuple length
impl<T1, T2> TypeDescription for (T1, T2)
where
    // NIT: Why is static needed? :confused:
    T1: TypeDescription + 'static,
    T2: TypeDescription + 'static,
{
    fn type_desc() -> ValueDesc {
        ValueDesc::Tuple {
            type_id: TypeId::of::<Self>(),
            values: vec![T1::type_desc(), T2::type_desc()],
        }
    }
}
impl<T> TypeDescription for Vec<T>
where
    T: TypeDescription,
{
    fn type_desc() -> ValueDesc {
        todo!()
    }
}
impl<T> TypeDescription for Option<T>
where
    // NIT: Why is static needed? :confused:
    T: TypeDescription + 'static,
{
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "Option",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<T>()],
        }
    }
}
impl<K, V> TypeDescription for BTreeMap<K, V>
where
    // NIT: Why is static needed? :confused:
    K: TypeDescription + 'static,
    V: TypeDescription + 'static,
{
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "BTreeMap",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<K>(), ValueDesc::of::<V>()],
        }
    }
}
impl<T> TypeDescription for BTreeSet<T>
where
    // NIT: Why is static needed? :confused:
    T: TypeDescription + 'static,
{
    fn type_desc() -> ValueDesc {
        ValueDesc::Struct {
            name: "BTreeSet",
            type_id: TypeId::of::<Self>(),
            values: vec![ValueDesc::of::<T>()],
        }
    }
}
