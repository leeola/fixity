//! A series of primitive data types for interacting with the Fixity store.

pub mod appendlog;
pub mod bytes;
pub mod commitlog;
pub mod prollylist;
pub mod prollytree;
pub use {
    crate::cache::Structured as Object,
    appendlog::AppendLog,
    bytes::{Create as BytesCreate, Read as BytesRead},
    commitlog::CommitLog,
};
