//! Shared fixtures for the delivery verification test crates.

pub const SCAN: &str = "fixture-scan";

pub fn create_temp_dir(prefix: &str) -> std::io::Result<tempfile::TempDir> {
    tempfile::Builder::new().prefix(prefix).tempdir()
}
