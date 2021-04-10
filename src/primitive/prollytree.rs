// pub mod cursor_create;
// pub mod cursor_read;
// pub mod cursor_update;
// pub mod lru_read;
pub mod refimpl;
pub mod roller;
// cursor_create::CursorCreate,
// cursor_read::CursorRead,
// lru_read::LruRead,
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        value::{Addr, KeyOwned, ValueOwned},
        Error,
    },
    rkyv::std_impl::ArchivedVec,
    std::ops::Deref,
};
pub(crate) const ONE_LEN_BLOCK_WARNING: &str =
    "writing key & value that exceeds block size, this is highly inefficient";
/// An alias to a [`Node`] with owned parameters.
pub type NodeOwned = Node<Vec<(KeyOwned, Addr)>, Vec<(KeyOwned, ValueOwned)>>;
pub type ArchivedNode =
    Node<rkyv::Archived<Vec<(KeyOwned, Addr)>>, rkyv::Archived<Vec<(KeyOwned, ValueOwned)>>>;
/// The lowest storage block within Fixity, a Node within a Prolly Tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Node<B, L> {
    Branch(B),
    Leaf(L),
}
/*
pub enum NodeBL<B, L> {
    Branch(B),
    Leaf(L),
}
impl<B, L> NodeBL<B, L> {
    /// Return the key for this whole node, aka the first element's key.
    pub fn key<'a, K: 'a, V: 'a, A: 'a>(&'a self) -> Option<&K>
    where
        B: Deref<Target = [(K, V)]>,
        L: Deref<Target = [(K, A)]>,
    {
        match self {
            Self::Branch(v) => v.get(0).map(|(k, _)| k),
            Self::Leaf(v) => v.get(0).map(|(k, _)| k),
        }
    }
    /// Len of the underlying vec.
    pub fn len(&self) -> usize {
        match self {
            Self::Branch(v) => v.len(),
            Self::Leaf(v) => v.len(),
        }
    }
    /// Whether or not the underlying vec is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Branch(v) => v.is_empty(),
            Self::Leaf(v) => v.is_empty(),
        }
    }
}
*/
impl NodeOwned {
    /// Consume self and return the key for this whole node, aka the first element's key.
    pub fn into_key(self) -> Option<KeyOwned> {
        match self {
            Self::Branch(mut v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v.swap_remove(0).0)
                }
            },
            Self::Leaf(mut v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v.swap_remove(0).0)
                }
            },
        }
    }
}
mod node_rkyv_impl {
    use super::{ArchivedNode, Node, NodeOwned};
    pub enum NodeResolver<B, L>
    where
        B: rkyv::Archive,
        L: rkyv::Archive,
    {
        Branch(rkyv::Resolver<B>),
        Leaf(rkyv::Resolver<L>),
    }
    const _: () =
        {
            use core::marker::PhantomData;
            use rkyv::{offset_of, Archive};
            #[repr(u8)]
            enum ArchivedTag {
                Branch,
                Leaf,
            }
            #[repr(C)]
            struct ArchivedVariantBranch<B, L>(ArchivedTag, rkyv::Archived<B>, PhantomData<(B, L)>)
            where
                B: rkyv::Archive,
                L: rkyv::Archive;
            #[repr(C)]
            struct ArchivedVariantLeaf<B, L>(ArchivedTag, rkyv::Archived<L>, PhantomData<(B, L)>)
            where
                B: rkyv::Archive,
                L: rkyv::Archive;
            impl<B, L> Archive for Node<B, L>
            where
                B: rkyv::Archive,
                L: rkyv::Archive,
            {
                type Archived = Node<B, L>;
                type Resolver = NodeResolver<B, L>;
                fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
                    match resolver {
                        NodeResolver::Branch(resolver_0) => {
                            if let Node::Branch(self_0) = self {
                                Self::Archived::Branch(self_0.resolve(
                                    pos + offset_of!(ArchivedVariantBranch<B,L>, 1),
                                    resolver_0,
                                ))
                            } else {
                                {
                                    // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                    //::std::rt::begin_panic(
                                    panic!("enum resolver variant does not match value variant",)
                                }
                            }
                        },
                        NodeResolver::Leaf(resolver_0) => {
                            if let Node::Leaf(self_0) = self {
                                Node::Leaf(self_0.resolve(
                                    pos + offset_of!(ArchivedVariantLeaf<B,L>, 1),
                                    resolver_0,
                                ))
                            } else {
                                {
                                    // I'm unfamiliar with begin_panic.. disabling.. :sus:
                                    //::std::rt::begin_panic(
                                    panic!("enum resolver variant does not match value variant",)
                                }
                            }
                        },
                    }
                }
            }
        };
    const _: () = {
        use rkyv::{Archive, Fallible, Serialize};
        impl<__S: Fallible + ?Sized, B, L> Serialize<__S> for Node<B, L>
        where
            B: rkyv::Serialize<__S>,
            L: rkyv::Serialize<__S>,
        {
            fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                Ok(match self {
                    Self::Branch(_0) => {
                        NodeResolver::Branch(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                    Self::Leaf(_0) => {
                        NodeResolver::Leaf(Serialize::<__S>::serialize(_0, serializer)?)
                    },
                })
            }
        }
    };
    const _: () = {
        use rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<__D: Fallible + ?Sized, B, L> Deserialize<Node<B, L>, __D> for Archived<Node<B, L>>
        where
            B: Archive,
            Archived<B>: Deserialize<B, __D>,
            L: Archive,
            Archived<L>: Deserialize<L, __D>,
        {
            fn deserialize(&self, deserializer: &mut __D) -> Result<Node<B, L>, __D::Error> {
                Ok(match self {
                    Self::Branch(_0) => Node::<B, L>::Branch(_0.deserialize(deserializer)?),
                    Self::Leaf(_0) => Node::<B, L>::Leaf(_0.deserialize(deserializer)?),
                })
            }
        }
    };
}
