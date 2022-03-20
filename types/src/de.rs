pub enum FixityTypes {
    Uint32,
    GRegister,
    // LACounterDeserializer,
    // Possibly support extension types?
    // Ext(Option<T=()>),
}
// pub struct Deserializer {
//     pub type_: FixityTypes,
//     pub sub: Option<Box<Deserializer>>,
// }
pub enum Deserializer {
    Uint32,
    GRegister(GRegisterDeserializer),
}
pub struct GRegisterDeserializer {
    pub type_: Box<Deserializer>,
}
pub trait FixityType {
    // fn serialize(&self, _??) -> Vec<u8>;
    fn generics(&self) -> &'static [&'static str];
    fn types(&self) -> &'static [&'static str];
}

pub struct GRegister<T> {
    // u64 being the user ident for wip
    registers: std::collections::HashMap<u64, T>,
}
#[test]
fn foo() {}
