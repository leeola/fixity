// A hopefully short term unstable feature, since GATs are stablizing soon.
pub mod container;
pub mod contentid;
pub mod deser;
pub mod meta;
pub mod storage;
pub mod store;
pub use contentid::ContentHasher;
pub mod replicaid;
pub use meta::Meta;
pub mod type_desc;
pub use storage::{ContentStorage, MutStorage};
pub use store::Store;
pub mod content_store;
pub mod deser_store;
pub mod meta_store;
pub mod mut_store;
pub mod stores;
pub mod store_container {
    use std::ops::{Deref, DerefMut};

    use crate::{
        container::ContainerV4,
        contentid::Cid,
        store::StoreError,
        type_desc::{TypeDescription, ValueDesc},
    };
    use async_trait::async_trait;

    #[async_trait]
    pub trait StoreContainerExt {
        fn new_container<T: ContainerV4>(&self) -> WithStore<&Self, T>;
        async fn open<T: ContainerV4>(&self, cid: &Cid) -> WithStore<&Self, T>;
    }
    pub struct WithStore<S, T> {
        container: T,
        store: S,
    }
    impl<S, T> WithStore<S, T> {
        pub fn into_inner(self) -> T {
            self.container
        }
    }
    impl<S, T> Deref for WithStore<S, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.container
        }
    }
    impl<S, T> DerefMut for WithStore<S, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.container
        }
    }
    /// A trait to wrap a `Container` and passing in  
    #[async_trait]
    pub trait ContainerWithStore: Sized + Send + TypeDescription {
        type Container;
        fn deser_type_desc() -> ValueDesc;
        async fn save(&mut self) -> Result<Cid, StoreError>;
        async fn save_with_cids(&mut self, cids_buf: &mut Vec<Cid>) -> Result<(), StoreError>;
        async fn merge(&mut self, other: &Cid) -> Result<(), StoreError>;
        async fn diff(&mut self, other: &Cid) -> Result<Self::Container, StoreError>;
    }
    /*
    #[async_trait]
    impl<S, T> ContainerV4 for ContainerWithStore<S, T> {
        fn deser_type_desc() -> ValueDesc {
            T::deser_type_desc()
        }
        fn new_container<S: ContentStore<Cid>>(store: &S) -> Self {
            T::new_container()
        }
        async fn open<S: ContentStore<Cid>>(store: &S, cid: &Cid) -> Result<Self, StoreError> {
            self.container.open(store, cid).await
        }
        async fn save<S: ContentStore<Cid>>(&mut self, store: &S) -> Result<Cid, StoreError> {
            self.container.save(store)
        }
        async fn save_with_cids<S: ContentStore<Cid>>(
            &mut self,
            store: &S,
            cids_buf: &mut Vec<Cid>,
        ) -> Result<(), StoreError> {
            self.container.save_with_cids()
        }
        async fn merge<S: ContentStore<Cid>>(
            &mut self,
            store: &S,
            other: &Cid,
        ) -> Result<(), StoreError> {
            self.container.merge()
        }
        async fn diff<S: ContentStore<Cid>>(
            &mut self,
            store: &S,
            other: &Cid,
        ) -> Result<Self, StoreError> {
            self.container.diff()
        }
    }
    */
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
