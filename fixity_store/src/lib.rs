// A hopefully short term unstable feature, since GATs are stablizing soon.
#![feature(generic_associated_types)]

pub mod cid;
pub mod storage;
pub mod store;
pub mod content {
    use {
        crate::cid::{ContentHasher, Hasher},
        async_trait::async_trait,
    };
    pub type Error = ();
    // #[async_trait]
    // pub trait Content<S, H = Hasher>: Sized + Send
    // where
    //     H: ContentHasher,
    // {
    //     async fn load(store: &S, cid: &H::Cid) -> Result<Self, Error>;
    //     async fn save(&self, store: &S) -> Result<H::Cid, Error>;
    //     async fn save_with_cids(&self, store: &S, cid_buf: &mut Vec<H::Cid>) -> Result<(), Error>
    //     where
    //         S: Sync,
    //     {
    //         let cid = self.save(store).await?;
    //         cid_buf.push(cid);
    //         Ok(())
    //     }
    // }
}

use std::fmt::Debug;
struct Cont;
#[async_trait::async_trait]
impl Container for Cont {
    type Deser = Json;
    async fn get<T>(&self) -> Repr<T, Self::Deser>
    where
        T: DeserializeRefGats<Self::Deser>,
    {
        todo!()
    }
}
struct Json;
#[async_trait::async_trait]
pub trait Container {
    type Deser;
    async fn get<T>(&self) -> Repr<T, Self::Deser>
    where
        T: DeserializeGats<Self::Deser>;
}
pub struct Repr<T, D>
where
    T: DeserializeGats<D>,
{
    _t: std::marker::PhantomData<T>,
    _d: std::marker::PhantomData<D>,
}
#[async_trait::async_trait]
pub trait DeserializeGats<Deser>: DeserializeRefGats<Deser> + Sized {
    fn deserialize_owned(buf: &[u8]) -> Result<Self, Error>;
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error>;
}
impl<D, T> DeserializeGats<D> for T
where
    T: DeserializeRefGats<D> + serde::de::DeserializeOwned,
    for<'a> T::Ref<'a>: serde::Deserialize<'a>,
{
    fn deserialize_owned(buf: &[u8]) -> Result<Self, Error> {
        todo!()
    }
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error> {
        todo!()
    }
}
pub trait DeserializeRefGats<Deser> {
    type Ref<'a>;
}
impl<D> DeserializeRefGats<D> for String {
    type Ref<'a> = &'a str;
}
#[tokio::test]
async fn call_foo() {
    foo(Cont).await
}
async fn foo<C: Container>(_c: C)
where
    String: DeserializeGats<C::Deser>,
    for<'a> <String as DeserializeRefGats<C::Deser>>::Ref<'a>: Debug,
{
}

pub use cid::ContentHasher;
//    store::{Repr, Store},
pub type Error = ();

pub mod deser {
    pub use self::{rkyv::Rkyv, serde_json::SerdeJson};
    pub type Error = crate::Error;
    pub trait Serialize<Deser> {
        // NIT: It would be nice if constructing an Arc<[u8]> from this was more clean.
        type Bytes: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
        fn serialize(&self) -> Result<Self::Bytes, Error>;
    }
    pub trait DeserializeRef<Deser> {
        type Ref<'a>;
    }
    pub trait Deserialize<Deser>: DeserializeRef<Deser> + Sized {
        fn deserialize_owned(buf: &[u8]) -> Result<Self, Error>;
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error>;
    }
    mod serde_json {
        use super::{Deserialize, DeserializeRef, Error, Serialize};
        #[derive(Debug, Default)]
        pub struct SerdeJson;
        impl<T> Serialize<SerdeJson> for T
        where
            T: serde::Serialize,
        {
            type Bytes = Vec<u8>;
            fn serialize(&self) -> Result<Self::Bytes, Error> {
                let v = serde_json::to_vec(&self).unwrap();
                Ok(v)
            }
        }
        impl<T> Deserialize<SerdeJson> for T
        where
            T: DeserializeRef<SerdeJson> + serde::de::DeserializeOwned,
            for<'a> T::Ref<'a>: serde::Deserialize<'a>,
        {
            fn deserialize_owned(buf: &[u8]) -> Result<Self, Error> {
                let self_ = serde_json::from_slice(buf.as_ref()).unwrap();
                Ok(self_)
            }
            fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error> {
                let ref_ = serde_json::from_slice(buf.as_ref()).unwrap();
                Ok(ref_)
            }
        }
        // TODO: macro this.
        impl DeserializeRef<SerdeJson> for String {
            type Ref<'a> = &'a str;
        }
    }
    mod rkyv {
        use {
            super::{Deserialize, DeserializeRef, Error, Serialize},
            rkyv::{
                ser::serializers::AllocSerializer, AlignedVec, Archive,
                Deserialize as RkyvDeserialize, Infallible, Serialize as RkyvSerialize,
            },
        };
        #[derive(Debug, Default)]
        pub struct Rkyv;
        // NIT: Make the buffer size configurable..?
        impl<T> Serialize<Rkyv> for T
        where
            T: Archive + RkyvSerialize<AllocSerializer<256>> + Send + Sync + 'static,
            T::Archived: RkyvDeserialize<T, Infallible>,
        {
            type Bytes = AlignedVec;
            fn serialize(&self) -> Result<Self::Bytes, Error> {
                let aligned_vec = rkyv::to_bytes::<_, 256>(self).unwrap();
                Ok(aligned_vec)
            }
        }
        impl<T> DeserializeRef<Rkyv> for T
        where
            T: Archive,
            T::Archived: 'static,
        {
            type Ref<'a> = &'a T::Archived;
        }
        impl<T> Deserialize<Rkyv> for T
        where
            for<'a> T: Archive + DeserializeRef<Rkyv, Ref<'a> = &'a T::Archived>,
            T::Archived: RkyvDeserialize<T, Infallible>,
        {
            fn deserialize_owned(buf: &[u8]) -> Result<Self, Error> {
                let archived = Self::deserialize_ref(buf)?;
                let t: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
                Ok(t)
            }
            fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, Error> {
                // TODO: Feature gate and type gate. Or maybe make an Rkyv and RkyvUnsafe type?
                let archived = unsafe { rkyv::archived_root::<T>(buf) };
                Ok(archived)
            }
        }
        #[test]
        fn rkyv_io() {
            use super::Deserialize;
            let buf = Serialize::<Rkyv>::serialize(&String::from("foo"))
                .unwrap()
                .into_vec();
            dbg!(&buf);
            let s = <String as Deserialize<Rkyv>>::deserialize_ref(buf.as_slice()).unwrap();
            assert_eq!(s, "foo");
            let s = <String as Deserialize<Rkyv>>::deserialize_owned(buf.as_slice()).unwrap();
            assert_eq!(s, "foo");
        }
    }
}

/*
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    Addr,
    Uint32(u32),
    String(String),
}
pub enum Type {
    GCounter,
    CascadingCounter(Box<Type>),
}
pub enum Struct {
    GCounter(GCounter),
    LACounter(LACounter),
}
pub struct GCounter;
// Limited Alternate Counter
pub struct LACounter {
    limit: usize,
    inner: LACounterInner,
}
struct LACounterDeserializer;
struct LACounterInner {
    counter: GCounter,
    // alternate: Result<GCounter, Box<LACounterInner>>,
    alternate: Box<dyn Counter>,
}
pub trait Counter: FixityType {}
*/

pub trait FixityType {
    // fn serialize(&self, _??) -> Vec<u8>;
    fn generics(&self) -> &'static [&'static str];
    fn types(&self) -> &'static [&'static str];
}
// pub trait Deser: Sized {
//     fn de(t: &T) -> Vec<u8>;
// }
