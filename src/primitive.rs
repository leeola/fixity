//! A series of primitive data types for interacting with the Fixity store.

pub mod appendlog;
pub mod bytes;
pub mod chain;
pub mod commitlog;
pub mod prollylist;
pub mod prollytree;
use crate::{
    value::{Addr, Key},
    Error,
};
pub use {
    appendlog::AppendLog,
    bytes::{Create as BytesCreate, Read as BytesRead},
    commitlog::{Commit, CommitLog},
};

// TODO: kill
#[async_trait::async_trait]
pub trait Flush {
    async fn flush(&mut self) -> Result<Addr, Error>;
}
#[async_trait::async_trait]
pub trait Build {
    type Primitive;
    async fn build(self, addr: Option<Addr>) -> Result<Self::Primitive, Error>;
}
#[async_trait::async_trait]
pub trait InsertAddr {
    async fn insert_addr(&mut self, key: Key, addr: Addr) -> Result<(), Error>;
}
#[async_trait::async_trait]
pub trait GetAddr {
    async fn get_addr(&self, key: Key) -> Result<Option<Addr>, Error>;
}
