use {
    crate::{
        error::{InternalError, TypeError},
        primitive::prollylist::refimpl,
        storage::{StorageRead, StorageWrite},
        value::Value,
        Addr, Error,
    },
    fastcdc::{Chunk, FastCDC},
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite},
};
const CDC_MIN: usize = 1024 * 16;
const CDC_AVG: usize = 1024 * 32;
const CDC_MAX: usize = 1024 * 64;
pub struct Read<'s, S> {
    storage: &'s S,
    addr: Addr,
}
impl<'s, S> Read<'s, S> {
    pub fn new(storage: &'s S, addr: Addr) -> Self {
        Self { storage, addr }
    }
    pub async fn read<W>(&self, mut w: W) -> Result<u64, Error>
    where
        S: StorageRead,
        W: AsyncWrite + Unpin + Send,
    {
        let values = {
            let tree = refimpl::Read::new(self.storage, self.addr.clone());
            tree.to_vec().await?
        };
        let mut total_bytes = 0;
        for value in values {
            let addr = value.into_addr().ok_or(TypeError::UnexpectedValueVariant {
                at_segment: None,
                at_addr: None,
            })?;
            total_bytes += self.storage.read(addr, &mut w).await?;
        }
        Ok(total_bytes)
    }
}
pub struct Create<'s, S> {
    storage: &'s S,
    cdc_min: usize,
    cdc_avg: usize,
    cdc_max: usize,
}
impl<'s, S> Create<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self {
            storage,
            cdc_min: CDC_MIN,
            cdc_avg: CDC_AVG,
            cdc_max: CDC_MAX,
        }
    }
    pub async fn write<R>(&self, mut r: R) -> Result<Addr, Error>
    where
        S: StorageWrite,
        R: AsyncRead + Unpin + Send,
    {
        let addrs = {
            let mut addrs = Vec::new();
            let mut b = Vec::new();
            // I don't think len can ever differ from the Vec len..?
            let _ = r
                .read_to_end(&mut b)
                .await
                .map_err(|err| InternalError::Io(format!("{}", err)))?;
            // TODO: use chunked streaming once this [1] is fixed/merged, and when we impl a
            // proper streaming prollylist::Create, not the refimpl.
            // [1]: https://github.com/nlfiedler/fastcdc-rs/issues/3
            let chunker = FastCDC::new(&b, self.cdc_min, self.cdc_avg, self.cdc_max);
            for Chunk { offset, length } in chunker {
                let chunk = &b[offset..offset + length];
                let addr = Addr::hash(chunk);
                self.storage.write(addr.clone(), chunk).await?;
                addrs.push(Value::Addr(addr));
            }
            addrs
        };
        let tree = refimpl::Create::new(self.storage);
        tree.with_vec(addrs).await
    }
}
