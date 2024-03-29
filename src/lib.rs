// Lints.
#![warn(
    unsafe_code,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    clippy::complexity,
    clippy::perf,
    // clippy::pedantic
    // clippy::nursery
    // clippy::cargo,
    clippy::unwrap_used,
)]
// This warning makes less sense with enum-flavored error handling,
// which this library is using.
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

// pub mod bytes;
// pub mod config;
// pub mod core;
// pub mod error;
// pub mod fixity;
// pub mod map;
// pub mod path;
// pub mod value;
// pub use self::{
//     bytes::Bytes,
//     config::Config,
//     error::{Error, Result},
//     fixity::Fixity,
//     map::Map,
//     path::Path,
//     value::{Addr, Key, Scalar, Value},
// };
