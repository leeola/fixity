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
