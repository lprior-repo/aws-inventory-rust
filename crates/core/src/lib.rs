#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub mod catalog;
#[cfg(feature = "legacy")]
pub mod iam_trust;
pub mod inventory;
pub mod record;

#[cfg(feature = "legacy")]
mod legacy;
#[cfg(feature = "legacy")]
pub use legacy::*;
