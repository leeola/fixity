use {
    super::{Error, Repr, Store},
    crate::{
        cid::{ContentHasher, Hasher},
        storage::{memory::Memory, ContentStorage},
    },
    async_trait::async_trait,
    rkyv::{Archive, Deserialize, Serialize},
    std::marker::PhantomData,
};
// TODO: Back this store by an actual kv storage.
pub struct RkyvStore<Storage, H = Hasher> {
    hasher: H,
    storage: Storage,
}
impl<Storage, H> RkyvStore<Storage, H> {
    pub fn new(storage: Storage) -> Self
    where
        H: ContentHasher + Default,
        Storage: ContentStorage<H::Cid>,
    {
        Self {
            hasher: Default::default(),
            storage,
        }
    }
}
impl RkyvStore<Memory> {
    pub fn memory() -> Self {
        Self::new(Default::default())
    }
}
use rkyv::ser::serializers::AllocSerializer;
#[async_trait]
impl<S, H, T> Store<T, H> for RkyvStore<S, H>
where
    S: ContentStorage<H::Cid>,
    S::Content: From<Box<[u8]>>,
    H: ContentHasher,
    H::Cid: Copy,
    T: Archive + Serialize<AllocSerializer<256>> + Send + Sync + 'static,
    T::Archived: Deserialize<T, rkyv::Infallible>,
{
    type Repr = RkyvRef<S::Content, T>;
    async fn put(&self, t: T) -> Result<H::Cid, Error> {
        // FIXME: prototype unwraps.
        // NIT: Make the buffer size configurable. N on RkyvStore<N, ...>
        let aligned_vec = rkyv::to_bytes::<_, 256>(&t).unwrap();
        // NIT: is there a better way to do this? Converting to Box then Arc feels.. sad
        let slice = aligned_vec.into_boxed_slice();
        let cid = self.hasher.hash(&slice.as_ref());
        self.storage.write_unchecked(cid, slice).await?;
        Ok(cid)
    }
    async fn get(&self, cid: &H::Cid) -> Result<Self::Repr, Error> {
        // FIXME: prototype unwraps.
        let buf = self.storage.read_unchecked(cid).await?;
        Ok(RkyvRef {
            buf,
            _phantom: PhantomData,
        })
    }
}
pub struct RkyvRef<Buf, T> {
    buf: Buf,
    _phantom: PhantomData<T>,
}
impl<Buf, T> RkyvRef<Buf, T>
where
    Buf: AsRef<[u8]>,
{
    pub fn new(buf: Buf) -> Self {
        Self {
            buf,
            _phantom: PhantomData,
        }
    }
}
impl<Buf, T> Repr for RkyvRef<Buf, T>
where
    Buf: AsRef<[u8]>,
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
