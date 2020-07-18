use {
    crate::error::StoreError as Error,
    std::io::{BufWriter, Read, Write},
};
pub trait Store: StoreRead + StoreWrite {}
pub trait StoreRead {
    fn read<S>(&self, hash: S, w: &mut dyn Write) -> Result<(), Error>
    where
        S: AsRef<str>;
    fn read_string<S>(&self, hash: S) -> Result<String, Error>
    where
        S: AsRef<str>,
    {
        let mut buf = BufWriter::new(Vec::new());
        self.read(&hash, &mut buf)?;
        buf.flush().map_err(|err| Error::Io {
            hash: hash.as_ref().to_owned(),
            err,
        })?;
        let s = std::str::from_utf8(&buf.get_ref())
            .map_err(|err| Error::Utf8 {
                hash: hash.as_ref().to_owned(),
                err,
            })?
            .to_owned();
        Ok(s)
    }
}
pub trait StoreWrite {
    fn write(&self, hash: String, r: &mut dyn Read) -> Result<usize, Error>;
    fn write_string(&self, hash: String, s: String) -> Result<usize, Error> {
        let mut b = s.as_bytes();
        let len = b.len();
        self.write(hash, &mut b)?;
        Ok(len)
    }
}
