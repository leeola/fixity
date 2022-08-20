use async_trait::async_trait;
use fixity_store::{
    container::Container,
    contentid::ContentId,
    deser::{Deserialize, Rkyv, Serialize},
    store::StoreError,
    Store,
};
use rkyv::option::ArchivedOption;
use std::{fmt::Debug, ops::Deref};

pub struct AppendLog<Cid, T>(ArchivedStruct<Cid, T>)
where
    Cid: rkyv::Archive,
    T: rkyv::Archive;
impl<Cid, T> AppendLog<Cid, T>
where
    Cid: rkyv::Archive,
    T: rkyv::Archive,
{
    // pub fn inner(&self) -> &T {
    //     &self.0.data
    // }
    // pub fn inner_mut(&mut self) -> &mut T {
    //     &mut self.0.data
    // }
}
#[async_trait]
impl<Cid, T, S> Container<S> for AppendLog<Cid, T>
where
    S: Store<Cid = Cid, Deser = Rkyv>,
    Cid: ContentId,
    T: Container<S> + Sync,
    AppendLog<Cid, T>: Deserialize<S::Deser> + Serialize<S::Deser>,
    Cid: rkyv::Archive,
    T: rkyv::Archive,
    Cid::Archived: Send,
    T::Archived: Send,
{
    fn new() -> Self {
        Self(ArchivedStruct {
            // TODO: This .. is difficult. Holding an inner Archived means that all sub-fields
            // need to be, well, Archived. But this is intended to be a non-archived type,
            // a <T as Container>. I think i have to store a container separately... awkward.
            //
            // It works i suppose, but it's painful if i need to have two versions of the
            // data anyway.
            //
            // Ie if it's an ArchivedVec, well i'd have to keep a non-archived Vec<Ptr<T>> or
            // something locally. Not sure how this can work.
            data: T::new(),
            prev: ArchivedOption::None,
        })
    }
    async fn open(store: &S, cid: &S::Cid) -> Result<Self, StoreError> {
        store.get::<Self>(cid).await.unwrap();
        todo!()
    }
    async fn save(&self, store: &S) -> Result<S::Cid, StoreError> {
        todo!()
    }
    async fn save_with_cids(
        &self,
        store: &S,
        cids_buf: &mut Vec<S::Cid>,
    ) -> Result<(), StoreError> {
        todo!()
    }
}
#[derive(Debug, Default)]
#[cfg(feature = "deser_rkyv")]
#[derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
struct Struct<Cid, T> {
    data: T,
    // TODO: experiment with making this non-Option,
    // and doing a fallback () type load for the very first node.
    //
    // Doing this means a tax (two reads) on loading the very first node, but every following
    // node pays no tax and has no `Option` overhead on storage.
    prev: Option<Cid>,
}
#[cfg(feature = "deser_rkyv")]
mod rkyv_impls {
    use rkyv::option::ArchivedOption;

    use super::*;
    impl<Cid, T> Default for ArchivedStruct<Cid, T>
    where
        Cid: Default + rkyv::Archive,
        T: Default + rkyv::Archive,
        T::Archived: Default,
    {
        fn default() -> Self {
            Self {
                data: Default::default(),
                prev: ArchivedOption::None,
            }
        }
    }
    impl<Cid, T> Debug for ArchivedStruct<Cid, T>
    where
        Cid: Debug + rkyv::Archive,
        T: Debug + rkyv::Archive,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }
}
