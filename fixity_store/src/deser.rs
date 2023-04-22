pub use self::{rkyv::Rkyv, serde_json::SerdeJson};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserError {}
pub trait Serialize<Deser> {
    // NIT: It would be nice if constructing an Arc<[u8]> from this was more clean.
    type Bytes: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
    fn serialize(&self) -> Result<Self::Bytes, DeserError>;
}
pub trait DeserializeRef<Deser>: Sized {
    type Ref<'a>;
}
pub trait Deserialize<Deser>: DeserializeRef<Deser> + Sized {
    fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError>;
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError>;
}
mod serde_json {
    use super::{DeserError, Deserialize, DeserializeRef, Serialize};
    #[derive(Debug, Default)]
    pub struct SerdeJson;
    impl<T> Serialize<SerdeJson> for T
    where
        T: serde::Serialize,
    {
        type Bytes = Vec<u8>;
        fn serialize(&self) -> Result<Self::Bytes, DeserError> {
            let v = serde_json::to_vec(&self).unwrap();
            Ok(v)
        }
    }
    impl<T> Deserialize<SerdeJson> for T
    where
        T: DeserializeRef<SerdeJson> + serde::de::DeserializeOwned,
        for<'a> T::Ref<'a>: serde::Deserialize<'a>,
    {
        fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError> {
            let self_ = serde_json::from_slice(buf.as_ref()).unwrap();
            Ok(self_)
        }
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError> {
            let ref_ = serde_json::from_slice(buf.as_ref()).unwrap();
            Ok(ref_)
        }
    }
    // TODO: macro this.
    impl DeserializeRef<SerdeJson> for String {
        type Ref<'a> = &'a str;
    }
}
#[cfg(feature = "rkyv")]
pub mod rkyv {
    use super::{DeserError, Deserialize, DeserializeRef, Serialize};
    use rkyv::{
        ser::serializers::AllocSerializer, AlignedVec, Archive, Deserialize as RkyvDeserialize,
        Infallible, Serialize as RkyvSerialize,
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
        fn serialize(&self) -> Result<Self::Bytes, DeserError> {
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
        fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError> {
            let archived = Self::deserialize_ref(buf)?;
            let t: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(t)
        }
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError> {
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
    /// A utility func to use Rkyv deserialize for T with feature flags to control unsafe vs safe
    /// deserialization.
    pub fn deserialize_owned<T>(buf: &[u8]) -> Result<T, DeserError>
    where
        T: Archive,
        T::Archived: rkyv::Deserialize<T, Infallible>,
    {
        let archived = deserialize_ref::<T>(buf)?;
        let t: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(t)
    }
    /// A utility func to use Rkyv deserialize for T with feature flags to control unsafe vs safe
    /// deserialization.
    pub fn deserialize_ref<T: Archive>(buf: &[u8]) -> Result<&T::Archived, DeserError> {
        let archived = unsafe { rkyv::archived_root::<T>(buf) };
        Ok(archived)
    }
}
