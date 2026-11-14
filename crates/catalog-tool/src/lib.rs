//! Reproducible catalog tooling that does not depend on Python.
//!
//! The library surface lets callers drive packaging and the AWS model report
//! using the same implementations as the `catalog-tool` binary.
//! See [`overlay`] for the versioned, reviewed per-service overlay format.

pub mod classes;
pub mod endpoints;
pub mod names;
pub mod overlay;
pub mod package;
pub mod plans;
pub mod smithy;
