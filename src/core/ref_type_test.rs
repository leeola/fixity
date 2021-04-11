//! Delete me.

use {
    rkyv::{
        archived_value,
        de::deserializers::AllocDeserializer,
        ser::{serializers::WriteSerializer, Serializer},
        std_impl::{ArchivedString, ArchivedVec},
        Archive, Deserialize, Serialize,
    },
    std::borrow::Borrow,
};
pub trait AsAddrRef {
    fn as_addr_ref<'a>(&'a self) -> AddrRef<'a>;
}
pub trait AsScalarRef {
    fn as_scalar_ref<'a>(&'a self) -> ScalarRef<'a>;
}
pub trait AsValueRef {
    type Scalar: AsScalarRef;
    fn as_value_ref<'a>(&'a self) -> ValueRef<'a, Self::Scalar>;
}
impl<T> AsValueRef for &T
where
    T: AsValueRef,
{
    type Scalar = T::Scalar;
    fn as_value_ref<'a>(&'a self) -> ValueRef<'a, Self::Scalar> {
        (*self).as_value_ref()
    }
}
pub trait AsScalarSlice {
    type Scalar: AsScalarRef;
    fn as_scalar_slice<'a>(&'a self) -> &'a [Self::Scalar];
}
impl AsScalarSlice for Vec<ScalarOwned> {
    type Scalar = ScalarOwned;
    fn as_scalar_slice<'a>(&'a self) -> &'a [Self::Scalar] {
        self.as_ref()
    }
}
impl<T> AsScalarSlice for &T
where
    T: AsScalarSlice,
{
    type Scalar = T::Scalar;
    fn as_scalar_slice<'a>(&'a self) -> &'a [Self::Scalar] {
        (*self).as_scalar_slice()
    }
}
impl<T> AsScalarSlice for rkyv::std_impl::ArchivedVec<T>
where
    T: AsScalarRef,
{
    type Scalar = T;
    fn as_scalar_slice<'a>(&'a self) -> &'a [Self::Scalar] {
        self.as_ref()
    }
}
/*
impl<T, U> AsScalarRef for T
where
    T: std::ops::Deref<Target = U>,
    U: AsScalarRef,
{
    fn as_scalar_ref<'a>(&'a self) -> ScalarRef<'a> {
        self.as_scalar_ref()
    }
}
*/
impl<T> AsScalarRef for &T
where
    T: AsScalarRef,
{
    fn as_scalar_ref<'a>(&'a self) -> ScalarRef<'a> {
        (*self).as_scalar_ref()
    }
}
pub trait AsBranchRef {
    type Key: AsValueRef;
    type Addr: AsAddrRef;
    fn as_branch_ref(&self) -> &[(Self::Key, Self::Addr)];
}
impl AsBranchRef for Vec<(KeyOwned, Addr)> {
    type Key = KeyOwned;
    type Addr = Addr;
    fn as_branch_ref(&self) -> &[(Self::Key, Self::Addr)] {
        self.as_ref()
    }
}
impl AsBranchRef for ArchivedVec<(KeyArchived, ArchivedAddr)> {
    type Key = KeyArchived;
    type Addr = ArchivedAddr;
    fn as_branch_ref(&self) -> &[(Self::Key, Self::Addr)] {
        self.as_ref()
    }
}
pub trait AsNodeRef {
    type Scalar: AsScalarRef;
    fn as_node_ref<'a>(&'a self) -> NodeRef<'a, Self::Scalar>;
}
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Addr(pub [u8; 32]);
impl AsRef<Addr> for Addr {
    fn as_ref(&self) -> &Addr {
        &self
    }
}
impl AsAddrRef for Addr {
    fn as_addr_ref<'a>(&'a self) -> AddrRef<'a> {
        AddrRef(&self.0)
    }
}
impl AsAddrRef for ArchivedAddr {
    fn as_addr_ref<'a>(&'a self) -> AddrRef<'a> {
        AddrRef(&self.0)
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AddrRef<'a>(pub &'a [u8; 32]);
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Scalar<ADDR, STRING> {
    Addr(ADDR),
    Uint32(u32),
    String(STRING),
}
pub type ScalarOwned = Scalar<Addr, String>;
pub type ScalarRef<'a> = Scalar<AddrRef<'a>, &'a str>;
pub type ScalarArchived = Scalar<ArchivedAddr, ArchivedString>;
impl<ADDR, STRING> AsScalarRef for Scalar<ADDR, STRING>
where
    ADDR: AsAddrRef,
    STRING: AsRef<str>,
{
    fn as_scalar_ref<'a>(&'a self) -> ScalarRef<'a> {
        match self {
            Self::Addr(addr) => Scalar::Addr(addr.as_addr_ref()),
            Self::Uint32(i) => Scalar::Uint32(*i),
            Self::String(s) => Scalar::String(s.as_ref()),
        }
    }
}
impl<ADDR, STRING> AsScalarRef for ArchivedScalar<ADDR, STRING>
where
    ADDR: Archive,
    ADDR::Archived: AsAddrRef,
    STRING: Archive,
    STRING::Archived: Borrow<str>,
{
    fn as_scalar_ref<'a>(&'a self) -> ScalarRef<'a> {
        match self {
            Self::Addr(addr) => Scalar::Addr(addr.as_addr_ref()),
            Self::Uint32(i) => Scalar::Uint32(*i),
            Self::String(s) => Scalar::String(s.borrow()),
        }
    }
}
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Value<ADDR, STRING, VEC> {
    Addr(ADDR),
    Uint32(u32),
    String(STRING),
    Vec(VEC),
}
pub type ValueOwned = Value<Addr, String, Vec<ScalarOwned>>;
pub type ValueRef<'a, SCALAR> = Value<AddrRef<'a>, &'a str, &'a [SCALAR]>;
pub type ValueArchived = Value<ArchivedAddr, ArchivedString, ArchivedVec<ScalarArchived>>;
impl<ADDR, STRING, VEC> AsValueRef for Value<ADDR, STRING, VEC>
where
    ADDR: AsAddrRef,
    STRING: AsRef<str>,
    VEC: AsScalarSlice,
{
    type Scalar = VEC::Scalar;
    fn as_value_ref<'a>(&'a self) -> ValueRef<'a, Self::Scalar> {
        match self {
            Self::Addr(addr) => Value::Addr(addr.as_addr_ref()),
            Self::Uint32(i) => Value::Uint32(*i),
            Self::String(s) => Value::String(s.as_ref()),
            Self::Vec(v) => Value::Vec(v.as_scalar_slice()),
        }
    }
}
impl<ADDR, STRING, VEC> AsValueRef for ArchivedValue<ADDR, STRING, VEC>
where
    ADDR: Archive,
    ADDR::Archived: AsAddrRef,
    STRING: Archive,
    STRING::Archived: std::ops::Deref<Target = str>,
    VEC: Archive,
    VEC::Archived: AsScalarSlice,
{
    type Scalar = <<VEC as Archive>::Archived as AsScalarSlice>::Scalar;
    fn as_value_ref<'a>(&'a self) -> ValueRef<'a, Self::Scalar> {
        match self {
            Self::Addr(addr) => Value::Addr(addr.as_addr_ref()),
            Self::Uint32(i) => Value::Uint32(*i),
            Self::String(s) => Value::String(s.as_ref()),
            Self::Vec(v) => Value::Vec(v.as_scalar_slice()),
        }
    }
}
pub type Key<ADDR, STRING, VEC> = Value<ADDR, STRING, VEC>;
pub type KeyOwned = ValueOwned;
pub type KeyRef<'a, SCALAR> = ValueRef<'a, SCALAR>;
pub type KeyArchived = Key<ArchivedAddr, ArchivedString, ArchivedVec<ScalarArchived>>;
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Node<BranchVec, LeafVec> {
    Branch(BranchVec),
    Leaf(LeafVec),
}
pub type NodeOwned = Node<Vec<(KeyOwned, Addr)>, Vec<(KeyOwned, ValueOwned)>>;
pub type NodeRef<'a, SCALAR> =
    Node<&'a [(KeyRef<'a, SCALAR>, &'a Addr)], &'a [(KeyRef<'a, SCALAR>, ValueRef<'a, SCALAR>)]>;
/*
impl<BranchVec, LeafVec> AsNodeRef for Node<BranchVec, LeafVec>
    where
        BranchVec: AsRef<[(KeyRef<'a>, &'a Addr)]>,
        LeafVec: AsRef<[(KeyRef<'a>, ValueRef<'a>)]>,
{
    type Scalar = VEC::Scalar;
    fn as_node_ref<'a>(&'a self) -> NodeRef<'a, SCALAR>
    {
        match self {
            Self::Branch(v) => NodeRef::Branch(v.as_ref()),
            Self::Leaf(v) => NodeRef::Leaf(v.as_ref()),
        }
    }
}
*/
// impl AsRef<Node<ArchivedNode
impl<B, L> AsRef<Node<B, L>> for Node<B, L> {
    fn as_ref(&self) -> &Node<B, L> {
        &self
    }
}
/*
fn print_node<ADDR, STRING, VecScalar, BranchVec, LeafVec, T>(t: T)
where
    ADDR: AsRef<Addr>,
    STRING: AsRef<str>,
    VecScalar: AsRef<[Scalar<ADDR, STRING>]>,
    BranchVec: AsRef<[(Key<ADDR, STRING, VecScalar>, ADDR)]>,
    LeafVec: AsRef<[(Key<ADDR, STRING, VecScalar>, Value<ADDR, STRING, VecScalar>)]>,
    T: AsRef<Node<BranchVec, LeafVec>>,
{
    match t.as_ref() {
        Node::Branch(v) => v.as_ref().iter().for_each(|(key, addr)| {
            println!("branch({:?}, {:?})", key.string().as_ref(), addr.as_ref())
        }),
        Node::Leaf(v) => v.as_ref().iter().for_each(|(key, value)| {
            println!(
                "leaf({:?}, {:?})",
                key.string().as_ref(),
                value.string().as_ref()
            )
        }),
    }
}
*/
fn print_branch_tuple<T>(t: T)
where
    T: AsBranchRef,
{
    for (key, addr) in t.as_branch_ref() {
        println!("got addr, {:?}", addr.as_addr_ref());
        match key.as_value_ref() {
            Key::Addr(addr) => println!("got addr, {:?}", addr),
            Key::Uint32(i) => println!("got int, {}", i),
            Key::String(s) => println!("got string, {:?}", s),
            Key::Vec(v) => v
                .iter()
                .for_each(|s| println!("got vec scalar, {:?}", s.as_scalar_ref())),
        }
    }
}
fn print_value<T>(t: T)
where
    T: AsValueRef,
{
    match t.as_value_ref() {
        Value::Addr(addr) => println!("got addr, {:?}", addr),
        Value::Uint32(i) => println!("got int, {}", i),
        Value::String(s) => println!("got string, {:?}", s),
        Value::Vec(v) => v
            .iter()
            .for_each(|s| println!("got vec scalar, {:?}", s.as_scalar_ref())),
    }
}
fn print_scalar<T>(t: T)
where
    T: AsScalarRef,
{
    match t.as_scalar_ref() {
        Scalar::Addr(addr) => println!("got addr, {:?}", addr),
        Scalar::Uint32(i) => println!("got int, {}", i),
        Scalar::String(s) => println!("got string, {:?}", s),
    }
}
fn from_rkyv() {
    let owned = ScalarOwned::String(String::from("foo"));
    let mut serializer = WriteSerializer::new(Vec::new());
    let pos = serializer
        .serialize_value(&owned)
        .expect("failed to serialize value");
    let buf = serializer.into_inner();
    let archived = unsafe { archived_value::<ScalarOwned>(buf.as_ref(), pos) };
    print_scalar(&archived);
    print_scalar(archived);
    print_scalar(&owned);
    print_scalar(owned);

    let owned = ValueOwned::Vec(vec![
        ScalarOwned::String(String::from("foo")),
        ScalarOwned::String(String::from("var")),
    ]);
    let mut serializer = WriteSerializer::new(Vec::new());
    let pos = serializer
        .serialize_value(&owned)
        .expect("failed to serialize value");
    let buf = serializer.into_inner();
    let archived = unsafe { archived_value::<ValueOwned>(buf.as_ref(), pos) };
    print_value(&archived);
    print_value(archived);
    print_value(&owned);
    print_value(owned);

    // tuple for branch
    let owned = vec![(
        KeyOwned::Vec(vec![
            ScalarOwned::String(String::from("foo")),
            ScalarOwned::String(String::from("var")),
        ]),
        Addr([0; 32]),
    )];
    let mut serializer = WriteSerializer::new(Vec::new());
    let pos = serializer
        .serialize_value(&owned)
        .expect("failed to serialize value");
    let buf = serializer.into_inner();
    let archived = unsafe { archived_value::<Vec<ValueOwned>>(buf.as_ref(), pos) };
    print_branch_tuple(archived);
    print_branch_tuple(owned);

    /*
    let owned = Node::<Vec<(KeyOwned, Addr)>, _>::Leaf(vec![(
        KeyOwned::String(String::from("foo")),
        ValueOwned::String(String::from("bar")),
    )]);
    let mut serializer = WriteSerializer::new(Vec::new());
    let pos = serializer
        .serialize_value(&owned)
        .expect("failed to serialize value");
    let buf = serializer.into_inner();
    let archived = unsafe { archived_value::<NodeOwned>(buf.as_ref(), pos) };
    print_node(&owned);
    print_node(owned);
    //print_node(archived);
    */
}
#[test]
fn tests() {
    from_rkyv();
}
