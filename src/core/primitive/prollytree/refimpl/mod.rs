//! A [`crate::prolly`] reference implementation.
pub mod create;
pub mod read;
pub mod update;
pub use {
    create::Create,
    read::Read,
    update::{Change, Update},
};
