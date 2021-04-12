use {
    crate::{
        core::{
            cache::{CacheRead, CacheWrite, OwnedRef},
            deser::Deser,
            primitive::Object,
            storage::{Error, StorageRead, StorageWrite},
        },
        Addr,
    },
    log::debug,
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    tokio::io::{self, AsyncRead, AsyncWrite},
};
#[derive(Debug, Hash, Eq, PartialEq)]
enum CacheKey {
    Object(Addr),
    Bytes(Addr),
}
#[derive(Debug)]
enum CacheValue {
    Object(Arc<Object>),
    Bytes(Arc<Vec<u8>>),
}
pub struct DeserCache<S> {
    deser: Deser,
    storage: S,
    // TODO: use an LRU or something useful. This is just a simple test of Caching + Deser.
    // TODO: use a RwLock here. Or ideally a lock-free data structure.
    cache: Mutex<HashMap<CacheKey, CacheValue>>,
}
impl<S> DeserCache<S> {
    pub fn new(storage: S) -> Self {
        Self {
            // TODO: use a passed in deser.
            deser: Deser::default(),
            storage,
            cache: Mutex::new(HashMap::new()),
        }
    }
}
#[async_trait::async_trait]
impl<S> CacheRead for DeserCache<S>
where
    S: StorageRead + Send,
{
    type OwnedRef = Arc<Object>;
    async fn read_unstructured<A, W>(&self, addr: A, mut w: W) -> Result<u64, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let addr_ref = addr.as_ref();
        let buf = {
            let cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            cache
                // TODO: impl Borrow for CacheKey, to avoid this Addr clone.
                .get(&CacheKey::Bytes(addr_ref.clone()))
                .and_then(|k| match k {
                    CacheValue::Bytes(buf) => Some(Arc::clone(&buf)),
                    CacheValue::Object(_) => None,
                })
        };
        if let Some(buf) = buf {
            let len = io::copy(&mut buf.as_slice(), &mut w).await?;
            return Ok(len);
        }
        // we could have a concurrency issue here, where we read from storage twice.
        // This is low-risk (ie won't corrupt data/etc), and should be tweaked based on
        // what results in better performance.
        // Optimizing for duplicate cache inserts vs holding the lock longer.
        // Possibly even keeping some type of LockState to have short lock length?
        // /shrug, bench concern for down the road.
        let buf = {
            let mut buf = Vec::new();
            let _: u64 = StorageRead::read(&self.storage, addr_ref.clone(), &mut buf).await?;
            Arc::new(buf)
        };
        {
            let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            let prev = cache.insert(
                CacheKey::Bytes(addr_ref.clone()),
                CacheValue::Bytes(Arc::clone(&buf)),
            );
            if prev.is_some() {
                debug!("cache inserted twice, needless storage read");
            }
        }
        let len = io::copy(&mut buf.as_slice(), &mut w).await?;
        Ok(len)
    }
    async fn read_structured<A>(&self, addr: A) -> Result<Self::OwnedRef, Error>
    where
        A: AsRef<Addr> + Into<Addr> + Send,
    {
        let addr_ref = addr.as_ref();
        {
            let cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            // TODO: impl Borrow for CacheKey, to avoid this Addr clone.
            if let Some(CacheValue::Object(obj)) = cache.get(&CacheKey::Object(addr_ref.clone())) {
                return Ok(Arc::clone(&obj));
            }
        }
        // we could have a concurrency issue here, where we read from storage twice.
        // This is low-risk (ie won't corrupt data/etc), and should be tweaked based on
        // what results in better performance.
        // Optimizing for duplicate cache inserts vs holding the lock longer.
        // Possibly even keeping some type of LockState to have short lock length?
        // /shrug, bench concern for down the road.
        let obj = {
            let buf = {
                let mut buf = Vec::new();
                let _: u64 = StorageRead::read(&self.storage, addr_ref.clone(), &mut buf).await?;
                buf
            };
            let obj = self
                .deser
                .from_slice::<Object>(&buf)
                .map_err(|err| Error::Unhandled {
                    message: format!("deser: {}", err),
                })?;
            Arc::new(obj)
        };
        {
            let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            let cache_value = cache.insert(
                CacheKey::Object(addr_ref.clone()),
                CacheValue::Object(Arc::clone(&obj)),
            );
            if cache_value.is_some() {
                debug!("cache inserted twice, needless storage read");
            }
        }
        Ok(obj)
    }
}
impl OwnedRef for Arc<Object> {
    type Ref = Self;
    fn as_ref_structured(&self) -> &Self::Ref {
        self
    }
    fn into_owned_structured(self) -> Object {
        use std::ops::Deref;
        (*self.deref()).clone()
    }
}
#[async_trait::async_trait]
impl<S> CacheWrite for DeserCache<S>
where
    S: StorageWrite + Send,
{
    async fn write_unstructured<R>(&self, mut r: R) -> Result<Addr, Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        let buf = {
            let mut buf = Vec::new();
            let _: u64 = io::copy(&mut r, &mut buf).await?;
            Arc::new(buf)
        };
        let addr = Addr::hash(buf.as_ref());
        let new_to_cache = {
            let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            cache
                .insert(
                    CacheKey::Bytes(addr.clone()),
                    CacheValue::Bytes(Arc::clone(&buf)),
                )
                .is_none()
        };
        // as an optimization, if it's already in the memory cache we should be able to ignore
        // writing it to storage.
        //
        // :WARN: If this fails the cache won't be invalidated, we could/should defer writing to
        // cache until after failure.
        if new_to_cache {
            let _: u64 = self
                .storage
                .write(addr.clone(), &mut buf.as_slice())
                .await?;
        }
        Ok(addr)
    }
    async fn write_structured<T>(&self, object: T) -> Result<Addr, Error>
    where
        T: Into<Object> + Send,
    {
        let object = object.into();
        let buf = Deser::default()
            .to_vec(&object)
            .map_err(|err| Error::Unhandled {
                message: format!("deser: {}", err),
            })?;
        let addr = Addr::hash(&buf);
        let new_to_cache = {
            let mut cache = self.cache.lock().map_err(|_| Error::Unhandled {
                message: "cache mutex poisoned".to_owned(),
            })?;
            cache
                .insert(
                    CacheKey::Object(addr.clone()),
                    CacheValue::Object(Arc::new(object)),
                )
                .is_none()
        };
        // as an optimization, if it's already in the memory cache we should be able to ignore
        // writing it to storage.
        //
        // :WARN: If this fails the cache won't be invalidated, we could/should defer writing to
        // cache until after failure.
        if new_to_cache {
            let _: u64 = self
                .storage
                .write(addr.clone(), &mut buf.as_slice())
                .await?;
        }
        Ok(addr)
    }
}
