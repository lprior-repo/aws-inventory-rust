//! Regional observation loading.

use std::path::Path;

use serde_json::Value;

use crate::HarnessError;

pub async fn for_scan(
    database: &Path,
    scan_id: &str,
) -> Result<(Vec<Value>, Vec<Value>, String), HarnessError> {
    Ok((Vec::new(), Vec::new(), String::from("ok")))
}