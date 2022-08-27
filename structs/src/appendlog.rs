use async_trait::async_trait;
use fixity_store::{
    container::Container,
    contentid::ContentId,
    deser::{Deserialize, Rkyv, Serialize},
    store::{Repr, StoreError},
    Store,
};
use rkyv::{option::ArchivedOption, Deserialize as RkyvDeserialize, Infallible};
use std::{fmt::Debug, ops::Deref};

pub struct AppendLog<S, Cid, T, D = Rkyv> {
    store: S,
    repr: OwnedRepr<Cid, T, D>,
}
enum OwnedRepr<Cid, T, D> {
    Owned(AppendNode<Cid, T>),
    Repr(Repr<AppendNode<Cid, Cid>, D>),
}
impl<'a, S, Cid, T> AppendLog<S, Cid, T, Rkyv>
where
    S: Store<Cid = Cid, Deser = Rkyv>,
    Cid: rkyv::Archive + 'static,
    T: Deserialize<Rkyv>,
    AppendNode<Cid, Cid>:
        Deserialize<Rkyv, Ref<'a> = &'a ArchivedAppendNode<Cid, Cid>> + Serialize<Rkyv>,
{
    pub async fn inner(&mut self) -> Result<&T, StoreError> {
        // NIT: this early check seems like overhead but lifetime errors were causing headaches
        // so this just works easier for a minor, possibly none, cost.
        if let OwnedRepr::Repr(repr) = &self.repr {
            let owned_node = {
                let ptr_node = repr.repr_to_owned().unwrap();
                // Leaving this dead code here, because it illustrates a usage of subfielding
                // that needs to be supported by `Repr` / Deserialize abstraction traits.
                // // TODO: Try and abstract this deserialize into part of the `Repr` impl.
                // let data_cid: Cid = ref_node
                //     .data
                //     .deserialize(&mut rkyv::Infallible)
                //     .unwrap()
                //     .into_inner();
                let data = self
                    .store
                    .get(&ptr_node.data)
                    .await?
                    .repr_to_owned()
                    .unwrap();
                drop(repr);
                AppendNode {
                    data,
                    prev: ptr_node.prev,
                }
            };
            self.repr = OwnedRepr::Owned(owned_node);
        }
        match &self.repr {
            OwnedRepr::Owned(node) => Ok(&node.data),
            // NIT: This unreachable makes me sad.
            OwnedRepr::Repr(_) => {
                unreachable!("variant assigned above")
            },
        }
    }
    // pub fn inner_mut(&mut self) -> &mut T {
    //     &mut self.0.data
    // }
}
#[async_trait]
impl<Cid, T, S, D> Container<S> for AppendLog<Cid, T, D>
where
    S: Store<Cid = Cid, Deser = D>,
    D: Send + Sync,
    Cid: ContentId,
    T: Container<S> + Serialize<S::Deser> + Sync,
    AppendNode<Cid, Cid>: Deserialize<S::Deser> + Serialize<S::Deser>,
{
    fn new() -> Self {
        Self(OwnedRepr::Owned(AppendNode {
            data: T::new(),
            prev: None,
        }))
    }
    async fn open(store: &S, cid: &S::Cid) -> Result<Self, StoreError> {
        let repr = store.get::<AppendNode<Cid, Cid>>(cid).await.unwrap();
        Ok(Self(OwnedRepr::Repr(repr)))
    }
    async fn save(&mut self, store: &S) -> Result<S::Cid, StoreError> {
        let owned_node = match &self.0 {
            OwnedRepr::Owned(node) => node,
            OwnedRepr::Repr(_) => return Err(StoreError::NotModified),
        };
        let data_cid = store.put(&owned_node.data).await?;
        let ref_node = AppendNode {
            data: data_cid,
            prev: owned_node.prev.clone(),
        };
        store.put(&ref_node).await
    }
    async fn save_with_cids(
        &mut self,
        store: &S,
        cids_buf: &mut Vec<S::Cid>,
    ) -> Result<(), StoreError> {
        let owned_node = match &self.0 {
            OwnedRepr::Owned(node) => node,
            OwnedRepr::Repr(_) => return Err(StoreError::NotModified),
        };
        store.put_with_cids(&owned_node.data, cids_buf).await?;
        // grab the last cid, as that is the one from above.
        let data_cid = cids_buf[cids_buf.len() - 1].clone();
        let ref_node = AppendNode {
            data: data_cid,
            prev: owned_node.prev.clone(),
        };
        store.put_with_cids(&ref_node, cids_buf).await
    }
}
#[derive(Debug, Default)]
#[cfg(feature = "deser_rkyv")]
#[derive(rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
struct AppendNode<Cid, T> {
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
    impl<Cid, T> Default for ArchivedAppendNode<Cid, T>
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
    impl<Cid, T> Debug for ArchivedAppendNode<Cid, T>
    where
        Cid: Debug + rkyv::Archive,
        T: Debug + rkyv::Archive,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }
}
