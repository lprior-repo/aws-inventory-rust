//! Regional assertion execution.

pub mod fixture;
pub mod observations;

use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::assertions::helpers::check as add_check;

pub async fn check(
    root: &Path,
    database: &Path,
    scan_id: &str,
    _seed_report: Option<&Value>,
    _region: Option<&str>,
) -> Result<Vec<Value>, HarnessError> {
    let mut checks = Vec::new();

    // Try to load fixtures
    let fixtures = match fixture::load_fixtures(root) {
        Ok((doc, sha)) => Some((doc, sha)),
        Err(_) => None,
    };

    add_check(
        &mut checks,
        "regional_fixtures_loadable",
        fixtures.is_some(),
        None,
    );

    // Try to read observations
    let (resources, edges, status) = observations::for_scan(database, scan_id).await.unwrap_or((
        Vec::new(),
        Vec::new(),
        String::from("absent"),
    ));

    add_check(
        &mut checks,
        "regional:readable_bounded_evidence",
        !resources.is_empty() || !edges.is_empty(),
        Some(&Value::from(status)),
    );

    Ok(checks)
}
