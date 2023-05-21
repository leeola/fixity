use crate::{
    content_store::ContentStore,
    contentid::{Cid, ContentId},
    deser::DeserError,
    store::StoreError,
};
use async_trait::async_trait;
use std::marker::PhantomData;

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
#[cfg(feature = "rkyv")]
mod rkyv {
    use super::{Deserialize, Serialize};
    use crate::deser;
    use rkyv::{ser::serializers::AllocSerializer, AlignedVec, Infallible};

    impl<T> Serialize for T
    where
        T: rkyv::Archive + rkyv::Serialize<AllocSerializer<256>>,
        T::Archived: rkyv::Deserialize<T, Infallible>,
    {
        type Bytes = AlignedVec;
        fn serialize(&self) -> Result<Self::Bytes, deser::DeserError> {
            let aligned_vec = rkyv::to_bytes::<_, 256>(self).unwrap();
            Ok(aligned_vec)
        }
    }
    impl<T> Deserialize for T
    where
        T: rkyv::Archive,
        for<'a> <Self as rkyv::Archive>::Archived: rkyv::Deserialize<T, Infallible> + 'a,
    {
        type Ref<'a> = &'a <Self as rkyv::Archive>::Archived;
        fn deserialize_owned(buf: &[u8]) -> Result<Self, deser::DeserError> {
            crate::deser::rkyv::deserialize_owned::<Self>(buf)
        }
        fn deserialize_ref(buf: &[u8]) -> Result<Self::Ref<'_>, deser::DeserError> {
            crate::deser::rkyv::deserialize_ref::<Self>(buf)
        }
    }
}
