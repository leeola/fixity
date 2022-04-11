use {
    super::{Addr, Error, ReprBorrow, ReprToOwned, Store},
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
pub struct RkyvStore<A = Addr>(Mutex<HashMap<A, Arc<[u8]>>>);
impl RkyvStore {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
use rkyv::ser::serializers::AllocSerializer;
#[async_trait::async_trait]
impl<A, T> Store<T, A> for RkyvStore<A>
where
    T: Archive + Serialize<AllocSerializer<256>> + Send + Sync + 'static,
    T::Archived: Deserialize<T, rkyv::Infallible>,
    A: TryFrom<Vec<u8>> + Clone + Hash + Eq + Send + Sync,
{
    type Repr = RkyvRef<T>;
    async fn put(&self, t: T) -> Result<A, Error> {
        // FIXME: prototype unwraps.
        // NIT: Make the buffer size configurable. N on RkyvStore<N, ...>
        let bytes = rkyv::to_bytes::<_, 256>(&t).unwrap();
        // NIT: is there a better way to do this? Converting to Box then Arc feels.. sad
        let buf = Arc::<[u8]>::from(bytes.into_boxed_slice());
        let addr: A = multihash::Code::Blake3_256
            .digest(buf.as_ref())
            .to_bytes()
            .try_into()
            .map_err(|_| ())?;
        self.0.lock().unwrap().insert(addr.clone(), Arc::from(buf));
        Ok(addr)
    }
    async fn get(&self, cid: &A) -> Result<Self::Repr, Error> {
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
impl<T> ReprToOwned<T> for RkyvRef<T>
where
    T: Archive,
    // TODO: If this is correct, move the D to the root RkyvStore.
    T::Archived: Deserialize<T, rkyv::Infallible>,
{
    fn repr_to_owned(&self) -> Result<T, Error> {
        let archived = self.repr_borrow()?;
        let t: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(t)
    }
}
impl<T, Borrowed> ReprBorrow<Borrowed> for RkyvRef<T>
where
    T: Archive<Archived = Borrowed>,
{
    fn repr_borrow(&self) -> Result<&Borrowed, Error> {
        // TODO: Feature gate and type gate. Eg impl `RkyvStore<T, UnsafeUsage>` for this.
        // where `UnsafeUsage` is a concrete type that this impl will be against. `SafeUsage`
        // being the only other impl.
        let archived = unsafe { rkyv::archived_root::<T>(self.buf.as_ref()) };
        Ok(archived)
    }
}
