//! Bounded inventory ingestion, durable scan jobs, API admission and snapshot delivery.
//!
//! SQLite is the inventory source of truth; graph plans and portable exports
//! project finalized scans. Callers own cancellation and drain their operations
//! before closing stores. No collector or AWS credentials are configured here.
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub mod export;
pub mod graph;
pub mod jobs;
pub mod limiter;
pub mod store;
