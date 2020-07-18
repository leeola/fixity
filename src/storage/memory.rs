use {
    crate::{
        error::StoreError as Error,
        store::{StoreRead, StoreWrite},
    },
    std::{
        collections::HashMap,
        io::{Read, Write},
        sync::{Arc, Mutex},
    },
};
#[derive(Debug, Default, Clone)]
pub struct Memory(Arc<Mutex<HashMap<String, Vec<u8>>>>);
impl Memory {
    pub fn new() -> Self {
        Self::default()
    }
}
impl StoreRead for Memory {
    fn read<S>(&self, hash: S, w: &mut dyn Write) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        let hash = hash.as_ref();
        let store = self.0.lock().map_err(|err| Error::Storage {
            hash: hash.into(),
            message: format!("unable to acquire store lock: {0}", err),
        })?;
        let r: &Vec<u8> = store.get(hash).ok_or_else(|| Error::NotFound {
            hash: hash.to_owned(),
        })?;
        w.write_all(&r).unwrap();
        Ok(())
    }
}
impl StoreWrite for Memory {
    fn write(&self, hash: String, r: &mut dyn Read) -> Result<usize, Error> {
        let mut b = Vec::new();
        r.read_to_end(&mut b).map_err(|err| Error::Io {
            hash: hash.clone(),
            err,
        })?;
        let len = b.len();
        self.0
            .lock()
            .map_err(|err| Error::Storage {
                hash: hash.clone(),
                message: format!("unable to acquire store lock: {0}", err),
            })?
            .insert(hash, b);
        Ok(len)
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    fn io() {
        let mem = Memory::default();
        let key = "foo";
        let io_in = "bar".to_owned();
        mem.write_string(key.into(), io_in.clone()).unwrap();
        let io_out = mem.read_string(key).unwrap();
        assert_eq!(io_out, io_in);
    }
}
