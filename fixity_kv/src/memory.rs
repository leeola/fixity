use {
    super::{Error, Read, Write},
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};
#[derive(Debug, Default, Clone)]
pub struct Memory(Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>);
impl Memory {
    pub fn new() -> Self {
        Self::default()
    }
}
