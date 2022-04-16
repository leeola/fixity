use {
    super::{Cid, Error, Repr, Store},
    multihash::MultihashDigest,
    rkyv::{Archive, Deserialize, Serialize},
    std::{
        collections::HashMap,
        hash::Hash,
        marker::PhantomData,
        sync::{Arc, Mutex},
    },
};
// TODO: Back this store by an actual kv storage.
pub struct RkyvStore<C = Cid>(Mutex<HashMap<C, Arc<[u8]>>>);
impl RkyvStore {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
use rkyv::ser::serializers::AllocSerializer;
#[async_trait::async_trait]
impl<C, T> Store<T, C> for RkyvStore<C>
where
    T: Archive + Serialize<AllocSerializer<256>> + Send + Sync + 'static,
    T::Archived: Deserialize<T, rkyv::Infallible>,
    C: TryFrom<Vec<u8>> + Clone + Hash + Eq + Send + Sync,
{
    type Repr = RkyvRef<T>;
    async fn put(&self, t: T) -> Result<C, Error> {
        // FIXME: prototype unwraps.
        // NIT: Make the buffer size configurable. N on RkyvStore<N, ...>
        let bytes = rkyv::to_bytes::<_, 256>(&t).unwrap();
        // NIT: is there a better way to do this? Converting to Box then Arc feels.. sad
        let buf = Arc::<[u8]>::from(bytes.into_boxed_slice());
        let addr: C = multihash::Code::Blake3_256
            .digest(buf.as_ref())
            .to_bytes()
            .try_into()
            .map_err(|_| ())?;
        self.0.lock().unwrap().insert(addr.clone(), Arc::from(buf));
        Ok(addr)
    }
    async fn get(&self, cid: &C) -> Result<Self::Repr, Error> {
        let buf = {
            let map = self.0.lock().unwrap();
            Arc::clone(&map.get(cid).unwrap())
        };
        Ok(RkyvRef {
            buf,
            _phantom: PhantomData,
        })
    }
}
pub struct RkyvRef<T> {
    buf: Arc<[u8]>,
    _phantom: PhantomData<T>,
}
impl<T> RkyvRef<T> {
    pub fn new(buf: Arc<[u8]>) -> Self {
        Self {
            buf,
            _phantom: PhantomData,
        }
    }
}
impl<T> Repr for RkyvRef<T>
where
    T: Archive,
    // TODO: Move the D to the root RkyvStore.
    T::Archived: Deserialize<T, rkyv::Infallible>,
{
    type Owned = T;
    type Borrow = T::Archived;
    fn repr_to_owned(&self) -> Result<T, Error> {
        let archived = self.repr_borrow()?;
        let t: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(t)
    }
    fn repr_borrow(&self) -> Result<&Self::Borrow, Error> {
        // TODO: Feature gate and type gate. Eg impl `RkyvStore<T, UnsafeUsage>` for this.
        // where `UnsafeUsage` is a concrete type that this impl will be against. `SafeUsage`
        // being the only other impl.
        let archived = unsafe { rkyv::archived_root::<T>(self.buf.as_ref()) };
        Ok(archived)
    }
}
