use {
    crate::{
        core::{
            cache::{CacheRead, CacheWrite},
            primitive::prollylist::refimpl,
        },
        error::{Internal as InternalError, Type as TypeError},
        Addr, Error, Value,
    },
    fastcdc::{Chunk, FastCDC},
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite},
};
const CDC_MIN: usize = 1024 * 16;
const CDC_AVG: usize = 1024 * 32;
const CDC_MAX: usize = 1024 * 64;
pub struct Read<'s, C> {
    cache: &'s C,
    addr: Addr,
}
impl<'s, C> Read<'s, C> {
    pub fn new(cache: &'s C, addr: Addr) -> Self {
        Self { cache, addr }
    }
    pub async fn read<W>(&self, mut w: W) -> Result<u64, Error>
    where
        C: CacheRead,
        W: AsyncWrite + Unpin + Send,
    {
        let values = {
            let tree = refimpl::Read::new(self.cache, self.addr.clone());
            tree.to_vec().await?
        };
        let mut total_bytes = 0;
        for value in values {
            let addr = value.into_addr().ok_or(TypeError::UnexpectedValueVariant {
                at_segment: None,
                at_addr: None,
            })?;
            total_bytes += self.cache.read_unstructured(addr, &mut w).await?;
        }
        Ok(total_bytes)
    }
}
pub struct Create<'s, C> {
    cache: &'s C,
    cdc_min: usize,
    cdc_avg: usize,
    cdc_max: usize,
}
impl<'s, C> Create<'s, C> {
    pub fn new(cache: &'s C) -> Self {
        Self {
            cache,
            cdc_min: CDC_MIN,
            cdc_avg: CDC_AVG,
            cdc_max: CDC_MAX,
        }
    }
    pub async fn write<R>(&self, mut r: R) -> Result<Addr, Error>
    where
        C: CacheWrite,
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
                let addr = self.cache.write_unstructured(chunk).await?;
                addrs.push(Value::Addr(addr));
            }
            addrs
        };
        let tree = refimpl::Create::new(self.cache);
        tree.with_vec(addrs).await
    }
}
