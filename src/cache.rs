pub mod archive_cache;
pub use archive_cache::ArchiveCache;
use {
    crate::{
        primitive::{appendlog, commitlog, prollylist, prollytree},
        storage::{Error, StorageRead, StorageWrite},
        value::{Key, KeyOwned, Scalar, Value, ValueOwned},
        Addr,
    },
    std::convert::TryFrom,
    tokio::io::{AsyncRead, AsyncWrite},
};
/// An enum of all possible structured data stored within Fixity.
///
/// Unstructured data, aka raw bytes, are not represented within this
/// enum.
///
/// Primarily used with [`CacheRead`] and [`CacheWrite`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug)]
#[repr(u8)]
pub enum Structured<B, L> {
    ProllyTreeNode(prollytree::Node<B, L>),
    /* ProllyListNode(prollylist::NodeOwned),
     * CommitLogNode(appendlog::LogNode<commitlog::CommitNode>), */
}
pub type StructuredOwned = Structured<Vec<(KeyOwned, Addr)>, Vec<(KeyOwned, ValueOwned)>>;
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait CacheRead: Sync {
    /// The structured data that the cache can borrow and return in [`CacheRead::read_struct`].
    ///
    /// This is an associated type to allow implementers to define borrowed and alternate
    /// versions of the [`Structured`] data type. Generally this type should reflect [`Structured`],
    /// but doesn't have to be identical.
    type OwnedRef: OwnedRef;
    async fn read_unstructured<A, W>(&self, addr: A, w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send,
        W: AsyncWrite + Unpin + Send;
    async fn read_structured<A>(&self, addr: A) -> Result<Self::OwnedRef, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send;
}
/*
pub trait OwnedRef {
    // Another lovely place to use GATs, whenever they hit stable, rather than returning a
    // &Self::Ref, implementers could return a Self::Ref<'a>, which would be neato.
    // type Ref<'a>;
    //
    // The lack of GATs mostly affects Serde-like low-cost deserialization, ie swapping
    // `Foo<String>` for `Foo<&'a str>`. So until then, Serde-like impls will
    // have to be less efficient and duplicate memory on first-read, even if using as_ref().
    type Ref;
    fn as_ref(&self) -> &Self::Ref;
    fn into_owned(self) -> Structured;
    // Add some helpers to get the concrete Ts back, to avoid
    // having to match the enums outside.
    // fn as_ref<T>(self) -> Result<&T::Variant, CacheError>
    // where T: VariantTo,
    // Self::Ref: VariantOf<T>;
    // fn into_owned<T>(self) -> Result<T, CacheError>;
}
*/
use std::ops::Deref;
pub trait OwnedRef {
    type AddrRef: AsRef<Addr>;
    type StringRef: AsRef<str>;
    type ScalarVecRef: Deref<Target = [Scalar<Self::AddrRef, Self::StringRef>]>;
    type ProllyTreeNodeBranchRef: Deref<
        Target = [(
            Key<Self::AddrRef, Self::StringRef, Self::ScalarVecRef>,
            Self::AddrRef,
        )],
    >;
    type ProllyTreeNodeLeafRef: Deref<
        Target = [(
            Key<Self::AddrRef, Self::StringRef, Self::ScalarVecRef>,
            Value<Self::AddrRef, Self::StringRef, Self::ScalarVecRef>,
        )],
    >;
    fn as_ref_structured(
        &self,
    ) -> Result<Structured<Self::ProllyTreeNodeBranchRef, Self::ProllyTreeNodeLeafRef>, Error>;
    fn into_structured(&self) -> Result<StructuredOwned, Error>;
    //  fn as_ref<T>(&self) -> Result<&<Self::Ref as RefFrom<T>>::Ref, Error>
    //  where
    //      Self::Ref: RefFrom<T>;
    //  fn into_owned<T>(self) -> Result<T, Error>
    //  where
    //      Self::Owned: OwnedFrom<T>;
}
pub trait RefFrom<T> {
    type Ref;
    fn ref_from(&self) -> Result<&Self::Ref, Error>;
}
pub trait OwnedFrom<T> {
    fn owned_from(self) -> Result<T, Error>;
}
impl OwnedFrom<prollytree::NodeOwned> for StructuredOwned {
    fn owned_from(self) -> Result<prollytree::NodeOwned, Error> {
        match self {
            Structured::ProllyTreeNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
// allowing name repetition to avoid clobbering a std Read or Write trait.
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait CacheWrite: Sync {
    async fn write_unstructured<R>(&self, r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send;
    async fn write_structured<T>(&self, structured: T) -> Result<Addr, Error>
    where
        T: Into<StructuredOwned> + Send;
}
/// A helper trait to allow a single `T` to return references to both a `Workspace` and
/// a `Cache`.
///
/// See [`Commit`](crate::Commit) for example usage.
pub trait AsCacheRef {
    type Cache: CacheRead + CacheWrite;
    fn as_cache_ref(&self) -> &Self::Cache;
}
impl From<prollytree::NodeOwned> for StructuredOwned {
    fn from(t: prollytree::NodeOwned) -> Self {
        Self::ProllyTreeNode(t)
    }
}
// impl From<prollylist::NodeOwned> for Structured {
//     fn from(t: prollylist::NodeOwned) -> Self {
//         Self::ProllyListNode(t)
//     }
// }
impl<B, L> TryFrom<Structured<B, L>> for prollytree::Node<B, L> {
    type Error = Error;
    fn try_from(t: Structured<B, L>) -> Result<Self, Error> {
        match t {
            Structured::ProllyTreeNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
/*
impl TryFrom<Structured> for prollylist::NodeOwned {
    type Error = Error;
    fn try_from(t: Structured) -> Result<Self, Error> {
        dbg!(&t);
        match t {
            Structured::ProllyListNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
impl TryFrom<Structured> for appendlog::LogNode<commitlog::CommitNode> {
    type Error = Error;
    fn try_from(t: Structured) -> Result<Self, Error> {
        dbg!(&t);
        match t {
            Structured::CommitLogNode(t) => Ok(t),
            // TODO: this deserves a unique error variant. Possibly a cache-specific error?
            _ => Err(Error::Unhandled {
                message: "misaligned cache types".to_owned(),
            }),
        }
    }
}
*/
/*
mod cache_rkyv_impl {
    use {
        super::{prollytree, Structured},
        crate::value::{Key, Value},
    };
    pub enum Structured<B, L> {
        ProllyTreeNode(prollytree::Node<B, L>),
    }
    #[repr(u8)]
    pub enum ArchivedStructured<B, L>
    where
        prollytree::Node<B, L>: rkyv::Archive,
    {
        ProllyTreeNode(rkyv::Archived<prollytree::Node<B, L>>),
    }
    pub enum StructuredResolver<B, L>
    where
        prollytree::Node<B, L>: rkyv::Archive,
    {
        ProllyTreeNode(rkyv::Resolver<prollytree::Node<B, L>>),
    }
    const _: () = {
        use core::marker::PhantomData;
        use rkyv::{offset_of, Archive};
        #[repr(u8)]
        enum ArchivedTag {
            ProllyTreeNode,
        }
        #[repr(C)]
        struct ArchivedVariantProllyTreeNode<B, L>(
            ArchivedTag,
            rkyv::Archived<prollytree::Node<B, L>>,
            PhantomData<(B, L)>,
        )
        where
            prollytree::Node<B, L>: rkyv::Archive;
        impl<B, L> Archive for Structured<B, L>
        where
            prollytree::Node<B, L>: rkyv::Archive,
        {
            type Archived = ArchivedStructured<B, L>;
            type Resolver = StructuredResolver<B, L>;
            fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                match resolver {
                    StructuredResolver::ProllyTreeNode(resolver_0) => {
                        if let Structured::ProllyTreeNode(self_0) = self {
                            ArchivedStructured::ProllyTreeNode(self_0.resolve(
                                pos + {
                                    let uninit = ::memoffset::__priv::mem::MaybeUninit::<
                                        ArchivedVariantProllyTreeNode<B, L>,
                                    >::uninit();
                                    let base_ptr: *const ArchivedVariantProllyTreeNode<B, L> =
                                        uninit.as_ptr();
                                    let field_ptr = {
                                        #[allow(clippy::unneeded_field_pattern)]
                                        let ArchivedVariantProllyTreeNode::<B, L> { 1: _, .. };
                                        let base = base_ptr;
                                        #[allow(unused_unsafe)]
                                        unsafe {
                                            {
                                                &raw const (*(base
                                                    as *const ArchivedVariantProllyTreeNode<B, L>))
                                                    .1
                                            }
                                        }
                                    };
                                    (field_ptr as usize) - (base_ptr as usize)
                                },
                                resolver_0,
                            ))
                        } else {
                            {
                                ::std::rt::begin_panic(
                                    "enum resolver variant does not match value variant",
                                )
                            }
                        }
                    },
                }
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Fallible, Serialize};
        impl<__S: Fallible + ?Sized, B, L> Serialize<__S> for Structured<B, L>
        where
            prollytree::Node<B, L>: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(match self {
                    Self::ProllyTreeNode(_0) => StructuredResolver::ProllyTreeNode(
                        Serialize::<__S>::serialize(_0, serializer)?,
                    ),
                })
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized, B, L> Deserialize<Structured<B, L>, __D> for Archived<Structured<B, L>>
        where
            prollytree::Node<B, L>: Archive,
            Archived<prollytree::Node<B, L>>: Deserialize<prollytree::Node<B, L>, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<Structured<B, L>, __D::Error> {
                Ok(match self {
                    Self::ProllyTreeNode(_0) => {
                        Structured::<B, L>::ProllyTreeNode(_0.deserialize(deserializer)?)
                    },
                })
            }
        }
    };
}
*/
