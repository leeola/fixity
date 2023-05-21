use thiserror::Error;

pub trait Serialize {
    // NIT: It would be nice if constructing an Arc<[u8]> from this was more clean.
    //
    // Also, keeping the AlignedVec from Rkyv would be really nice... this part of the API
    // is tough to keep compatible between Rkyv and Serde/etc.
    type Bytes: AsRef<[u8]> + Into<Vec<u8>> + Send + 'static;
    fn serialize(&self) -> Result<Self::Bytes, DeserError>;
}
pub trait Deserialize: Sized {
    type Ref<'a>;
    fn deserialize_owned(buf: &[u8]) -> Result<Self, DeserError>;
    fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, DeserError>;
}
#[derive(Error, Debug)]
pub enum DeserError {}

#[cfg(feature = "rkyv")]
pub mod rkyv {
    use super::{DeserError, Deserialize, Serialize};
    use crate::deser;
    use rkyv::{
        ser::serializers::AllocSerializer, AlignedVec, Archive, Deserialize as RkyvDeserialize,
        Infallible,
    };

    impl<T> Serialize for T
    where
        T: rkyv::Archive + rkyv::Serialize<AllocSerializer<256>>,
        T::Archived: rkyv::Deserialize<T, Infallible>,
    {
        type Bytes = AlignedVec;
        fn serialize(&self) -> Result<Self::Bytes, deser::DeserError> {
            serialize(self)
        }
    }
    impl<T> Deserialize for T
    where
        T: rkyv::Archive,
        for<'a> <Self as rkyv::Archive>::Archived: rkyv::Deserialize<T, Infallible> + 'a,
    {
        type Ref<'a> = &'a <Self as rkyv::Archive>::Archived;
        fn deserialize_owned(buf: &[u8]) -> Result<Self, deser::DeserError> {
            deserialize_owned::<Self>(buf)
        }
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, deser::DeserError> {
            deserialize_ref::<Self>(buf)
        }
    }

    /// A utility func to use Rkyv serialization for `T`.
    pub fn serialize<T>(t: &T) -> Result<AlignedVec, DeserError>
    where
        // NIT: Make the buffer size configurable..?
        T: rkyv::Serialize<AllocSerializer<256>>,
    {
        let aligned_vec = rkyv::to_bytes::<_, 256>(t).unwrap();
        Ok(aligned_vec)
    }
    /// A utility func to use Rkyv deserialize for `T` with feature flags to control unsafe vs safe
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
