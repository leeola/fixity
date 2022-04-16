use {
    super::{Cid, Error, Repr, Store},
    std::{
        any::Any,
        collections::HashMap,
        hash::Hash,
        marker::PhantomData,
        sync::{Arc, Mutex},
    },
};
pub struct AnyStore<C = Cid>(Mutex<HashMap<C, DynAny>>);
impl AnyStore {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
#[async_trait::async_trait]
impl<T, C> Store<T, C> for AnyStore<C>
where
    T: Any + Clone + Send + Sync + 'static,
    C: From<Cid> + Clone + Hash + Eq + Send + Sync,
{
    type Repr = AnyRepr<T>;
    async fn put(&self, t: T) -> Result<C, ()> {
        let key = C::from([0u8; 34]);
        self.0.lock().unwrap().insert(key.clone(), Arc::new(t));
        Ok(key)
    }
    async fn get(&self, cid: &C) -> Result<Self::Repr, ()> {
        let t = {
            let map = self.0.lock().unwrap();
            Arc::clone(&map.get(cid).unwrap())
        };
        Ok(AnyRepr {
            ref_: t,
            _phantom: PhantomData,
        })
    }
}
type DynAny = Arc<dyn Any + Send + Sync>;
pub struct AnyRepr<T> {
    ref_: DynAny,
    _phantom: PhantomData<T>,
}
impl<T> Repr for AnyRepr<T>
where
    T: Any + Clone,
{
    type Owned = T;
    type Borrow = T;
    fn repr_to_owned(&self) -> Result<T, Error> {
        self.ref_
            .downcast_ref()
            .map_or(Err(()), |t: &T| Ok(t.clone()))
    }
    fn repr_borrow(&self) -> Result<&Self::Borrow, Error> {
        self.ref_.downcast_ref().map_or(Err(()), |t: &T| Ok(t))
    }
}
