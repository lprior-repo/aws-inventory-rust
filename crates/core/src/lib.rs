#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub mod catalog;
pub mod inventory;
pub mod record;

#[cfg(feature = "legacy")]
mod legacy;
#[cfg(feature = "legacy")]
pub use legacy::*;
