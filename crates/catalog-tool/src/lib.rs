//! Reproducible catalog tooling that does not depend on Python.
//!
//! The library surface lets callers drive packaging and the AWS model report
//! using the same implementations as the `catalog-tool` binary.

pub mod classes;
pub mod endpoints;
pub mod names;
pub mod package;
pub mod plans;
pub mod smithy;
