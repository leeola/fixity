// NIT: I think this blanket allows incomplete features.. i'd like to just allow
// this one.
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
// Lints.
#![warn(
    unsafe_code,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic
    // clippy::nursery
    // clippy::cargo,
)]

pub mod bytes;
pub mod deser;
mod dir;
pub mod error;
pub mod fixity;
pub mod map;
pub mod misc;
pub mod path;
pub mod primitive;
pub mod storage;
pub mod value;
pub mod workspace;

pub use self::{
    bytes::Bytes,
    error::{Error, Result},
    fixity::{Commit, Fixity},
    map::Map,
    path::Path,
    storage::{Storage, StorageRead, StorageWrite},
    value::{Addr, Key, Value},
};
