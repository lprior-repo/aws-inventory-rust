//! Parent resource assertions for Cloud Control API coverage.

pub mod checks;
pub mod identity;
pub mod scope;
pub mod seed;

use std::path::Path;

use serde_json::Value;

use crate::assertions::helpers::check as add_check;
use crate::HarnessError;

pub async fn check(
    root: &Path,
    _database: &Path,
    _scan_id: &str,
    seed_report: Option<&Value>,
) -> Result<Vec<Value>, HarnessError> {
    let mut checks = Vec::new();

    // Try to load fixtures
    let fixtures = match seed::load_fixtures(root) {
        Ok((doc, sha)) => Some((doc, sha)),
        Err(_) => None,
    };

    add_check(
        &mut checks,
        "parent_fixtures_loadable",
        fixtures.is_some(),
        None,
    );

    if let Some((_fixture, sha)) = fixtures {
        // Check seed report hash if provided
        let reported_hash = seed_report
            .and_then(|sr| sr.get("fixture_sha256"))
            .and_then(Value::as_str);

        add_check(
            &mut checks,
            "seed_source_hash",
            reported_hash == Some(sha.as_str()),
            None,
        );

        // Check seed status
        let status = seed_report
            .and_then(|sr| sr.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("");

        add_check(
            &mut checks,
            "seed_status_complete",
            status == "complete",
            None,
        );
    }

    Ok(checks)
}