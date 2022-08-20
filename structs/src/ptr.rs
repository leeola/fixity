use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use fixity_store::Store;

// NIT: Is there something cheaper than Arc? Since
// i don't care about using the Rc portion of Arc.
pub struct Ptr<Cid, T>(Arc<PtrInner<Cid, T>>);
enum PtrInner<Cid, T> {
    Ptr { cid: Cid },
    Ref { cid: Cid, value: T },
    Mut { value: T },
}
pub struct Registry<Cid, Owner, T>((Owner, Resolver<Cid, T>));
impl<Cid, Owner, T> Registry<Cid, Owner, T> {
    pub fn new(owner: Owner) -> Self {
        Self((owner, Default::default()))
    }
}
impl<Cid, Owner, T> Deref for Registry<Cid, Owner, T> {
    type Target = (Owner, Resolver<Cid, T>);
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Cid, Owner, T> DerefMut for Registry<Cid, Owner, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub struct Resolver<Cid, T> {
    weak_ptrs: HashMap<Cid, Ptr<Cid, T>>,
}
impl<Cid, T> Resolver<Cid, T> {
    // pub async fn resolve<S: Store>(&mut self, store: &S, ptr: &Ptr<Cid, T>) -> &T {
    //     todo!()
    // }
    // pub async fn resolve_mut<S: Store>(&mut self, store: &S, ptr: &mut Ptr<Cid, T>) -> &mut T {
    //     todo!()
    // }
}
impl<Cid, T> Default for Resolver<Cid, T> {
    fn default() -> Self {
        Self {
            weak_ptrs: Default::default(),
        }
    }
}
