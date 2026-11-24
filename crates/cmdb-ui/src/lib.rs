//! cmdb-ui library target exposing DAG graph and layout types for
//! benchmarking. Binary-only modules (app shell, workers, templates)
//! remain in main.rs.

pub mod dag;
pub mod data;
pub mod frame_harness;
pub mod freshness;

// Declared here and not in `dag/mod.rs`: the binary compiles its own `dag` tree and does not use the layout yet.
#[path = "dag/hierarchy.rs"]
pub mod hierarchy;
