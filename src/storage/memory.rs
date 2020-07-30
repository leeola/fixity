use {
    super::{Error, StorageRead, StorageWrite},
    std::{
        collections::HashMap,
        io::{Read, Write},
        sync::{Arc, Mutex},
    },
};
#[derive(Debug, Default, Clone)]
pub struct Memory(Arc<Mutex<HashMap<String, String>>>);
impl Memory {
    pub fn new() -> Self {
        Self::default()
    }
}
impl StorageRead for Memory {
    fn read<S, W>(&self, hash: S, mut w: W) -> Result<(), Error>
    where
        S: AsRef<str>,
        W: Write,
    {
        let hash = hash.as_ref();
        let store = self.0.lock().map_err(|err| Error::Unhandled {
            message: format!("unable to acquire store lock: {0}", err),
        })?;
        let r: &String = store.get(hash).ok_or_else(|| Error::NotFound {
            hash: hash.to_owned(),
        })?;
        w.write_all(&r.as_bytes()).unwrap();
        Ok(())
    }
}
impl StorageWrite for Memory {
    fn write<S, R>(&self, hash: S, mut r: R) -> Result<usize, Error>
    where
        S: AsRef<str>,
        R: Read,
    {
        let hash = hash.as_ref();
        let mut b = Vec::new();
        r.read_to_end(&mut b).map_err(|err| Error::Io {
            hash: hash.to_owned(),
            err,
        })?;
        let len = b.len();
        let s = String::from_utf8(b).map_err(|err| Error::Unhandled {
            message: format!("{} is not valid utf8", hash),
        })?;
        self.0
            .lock()
            .map_err(|err| Error::Unhandled {
                message: format!("unable to acquire store lock: {0}", err),
            })?
            .insert(hash.to_owned(), s);
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
        mem.write_string(key, io_in.clone()).unwrap();
        let io_out = mem.read_string(key).unwrap();
        assert_eq!(io_out, io_in);
    }
}
