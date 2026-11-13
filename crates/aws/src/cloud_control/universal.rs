//! Catalog-wide Cloud Control drain over the pinned universal schema archive.
//!
//! Every archived type is accounted for: list-capable rows are attempted with
//! one ListResources cursor chain, while parent-dependent and unsupported rows
//! are reported as explicit skips. A provider `TypeNotFoundException` becomes a
//! persisted `unsupported_type` outcome; authorization, throttling and decode
//! failures remain ordinary scope failures.

use super::*;

mod outcomes;
mod selection;
pub(crate) use outcomes::collect;
pub use selection::{UniversalReport, digest, list_descriptors, report};

#[cfg(test)]
mod tests;
