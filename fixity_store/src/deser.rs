use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserError {}

#[cfg(feature = "rkyv")]
pub mod rkyv {
    use super::DeserError;
    use rkyv::{
        ser::serializers::AllocSerializer, AlignedVec, Archive, Deserialize as RkyvDeserialize,
        Infallible,
    };
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
