//! Parsing, identity/fingerprint, deduplication and malformed-input coverage.

mod support;

use cmdb_view::{Provenance, ScanStatus, Snapshot, ViewError};
use serde_json::json;

fn parse_mutated(
    mutate: impl FnOnce(&mut serde_json::Value),
) -> Result<Result<Snapshot, ViewError>, Box<dyn std::error::Error>> {
    let mut value = support::document();
    mutate(&mut value);
    Ok(Snapshot::parse(&serde_json::to_vec(&value)?))
}

#[test]
fn parses_header_identity_and_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let meta = snapshot.meta();
    assert_eq!(meta.artifact, "aws-inventory.snapshot");
    assert_eq!(meta.version, 1);
    assert_eq!(meta.provenance, Provenance::Normalized);
    assert_eq!(meta.scan_id, "scan-fixture");
    assert_eq!(meta.status, ScanStatus::Complete);
    assert_eq!(meta.started_at, 10);
    assert_eq!(meta.finished_at, 20);
    assert_eq!(meta.normalized_resources, 4);
    assert_eq!(meta.normalized_edges, 5);
    assert_eq!(meta.completed_scopes, 2);
    assert_eq!(meta.failed_scopes, 1);
    assert_eq!(meta.catalog_digest.as_deref(), Some("catalog-sha"));
    assert_eq!(meta.evidence_digest, None);
    assert_eq!(meta.fingerprint.len(), 64);
    assert!(meta.fingerprint().chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(meta.fingerprint(), snapshot.fingerprint());
    Ok(())
}

#[test]
fn fingerprint_is_stable_and_changes_with_scan_identity() -> Result<(), Box<dyn std::error::Error>>
{
    let first = support::snapshot()?;
    let second = Snapshot::parse(&support::bytes()?)?;
    assert_eq!(first.fingerprint(), second.fingerprint());
    let renamed = Snapshot::parse(&support::bytes_with_scan_id("scan-other")?)?;
    assert_ne!(first.fingerprint(), renamed.fingerprint());
    Ok(())
}

#[test]
fn orders_resources_and_edges_deterministically() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let ids: Vec<&str> = snapshot
        .resources()
        .iter()
        .map(|resource| resource.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            support::INSTANCE_ONE,
            support::INSTANCE_TWO,
            support::BUCKET,
            support::ROLE
        ]
    );
    let edges: Vec<(&str, &str, &str)> = snapshot
        .edges()
        .iter()
        .map(|edge| edge.sort_key())
        .collect();
    assert_eq!(
        edges,
        vec![
            (support::BUCKET, support::DANGLING, "references"),
            (support::INSTANCE_ONE, support::BUCKET, "references"),
            (support::INSTANCE_ONE, support::INSTANCE_TWO, "contains"),
            (support::INSTANCE_TWO, support::INSTANCE_ONE, "references"),
            (support::INSTANCE_TWO, support::ROLE, "references"),
        ]
    );
    Ok(())
}

#[test]
fn exposes_coverage_and_property_sizes() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let coverage = snapshot.coverage();
    assert_eq!(coverage.scopes.len(), 1);
    assert_eq!(coverage.failures.len(), 1);
    assert!(!coverage.scopes_truncated);
    let scope = coverage.scopes.first().ok_or("scope")?;
    assert_eq!(scope.account_id, support::ACCOUNT_A);
    assert!(scope.complete);
    assert!(!scope.failed);
    let failure = coverage.failures.first().ok_or("failure")?;
    assert_eq!(failure.category, "access_denied");
    assert_eq!(failure.message, "denied");
    assert!(
        snapshot
            .resources()
            .iter()
            .all(|resource| resource.properties_bytes == 9)
    );
    Ok(())
}

#[test]
fn deduplicates_all_observation_records() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = Snapshot::parse(&support::duplicate_bytes()?)?;
    assert_eq!(snapshot.meta().provenance, Provenance::AllObservations);
    assert_eq!(snapshot.resources().len(), 4);
    assert_eq!(snapshot.edges().len(), 5);
    Ok(())
}

#[test]
fn rejects_unsupported_artifact() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_mutated(|value| {
        if let Some(artifact) = value
            .get_mut("header")
            .and_then(|header| header.get_mut("artifact"))
        {
            *artifact = json!("other.snapshot");
        }
    })?;
    assert!(matches!(result, Err(ViewError::UnsupportedArtifact(_))));
    Ok(())
}

#[test]
fn rejects_unsupported_version() -> Result<(), Box<dyn std::error::Error>> {
    let result = parse_mutated(|value| {
        if let Some(version) = value
            .get_mut("header")
            .and_then(|header| header.get_mut("version"))
        {
            *version = json!(2);
        }
    })?;
    assert!(matches!(result, Err(ViewError::UnsupportedVersion(2))));
    Ok(())
}

#[test]
fn rejects_malformed_json() {
    assert!(matches!(
        Snapshot::parse(b"{\"header\":"),
        Err(ViewError::Json(_))
    ));
}

#[test]
fn rejects_unknown_record_tag_and_missing_identity() -> Result<(), Box<dyn std::error::Error>> {
    let unknown = parse_mutated(|value| {
        if let Some(record) = value
            .get_mut("records")
            .and_then(serde_json::Value::as_array_mut)
            .and_then(|records| records.first_mut())
            .and_then(|record| record.get_mut("record"))
        {
            *record = json!("mystery");
        }
    })?;
    assert!(matches!(unknown, Err(ViewError::Json(_))));
    let missing = parse_mutated(|value| {
        if let Some(resource) = value
            .get_mut("records")
            .and_then(serde_json::Value::as_array_mut)
            .and_then(|records| records.first_mut())
            .and_then(|record| record.get_mut("data"))
            .and_then(|data| data.get_mut("resource"))
            .and_then(serde_json::Value::as_object_mut)
        {
            resource.remove("id");
        }
    })?;
    assert!(matches!(missing, Err(ViewError::Json(_))));
    Ok(())
}
