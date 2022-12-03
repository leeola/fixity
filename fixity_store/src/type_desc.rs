use std::{any::TypeId, fmt::Display};

pub trait TypeDescription {
    fn type_desc() -> ValueDesc;
}
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
        value: Box<ValueDesc>,
        type_id: TypeId,
    },
    Array {
        value: Box<ValueDesc>,
        type_id: TypeId,
        len: usize,
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
    pub fn type_eq(&self, other: &Self) -> bool {
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
impl TypeDescription for i64 {
    fn type_desc() -> ValueDesc {
        ValueDesc::Number(TypeId::of::<Self>())
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
