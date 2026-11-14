use std::collections::BTreeSet;

use inventory_harness::emulator::{PINNED_COMMIT, PINNED_VERSION};

use super::*;

const EXPECTED_TOTAL: u64 = 1_881;
const EXPECTED_LIST_CAPABLE: u64 = 1_291;
const EXPECTED_SKIPPED_PARENT_REQUIRED: u64 = 410;
const EXPECTED_SKIPPED_UNSUPPORTED: u64 = 180;
const EXPECTED_ATTEMPTED: u64 = 1_291;
const EXPECTED_SUCCEEDED: u64 = 999;
const EXPECTED_UNSUPPORTED: u64 = 292;
const EXPECTED_FAILED: u64 = 0;
const FIXTURE: &str = "crates/harness/fixtures/universal-unsupported-aws-emulator-1.0.348.json";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the pinned emulator binary and a collector build"]
async fn given_owned_emulator_when_every_universal_type_is_scanned_then_partition_is_explicit()
-> TestResult {
    let work = tempfile::tempdir()?;
    let path = work.path().to_path_buf();
    let binary = emulator_binary()?;
    let emulator = tokio::task::spawn_blocking(move || Emulator::start(&binary, &path, 900))
        .await
        .map_err(|_| "emulator boot task panicked")??;
    let database = work.path().join("universal.sqlite");
    let database_text = text(&database)?;
    let report = scan(
        &database_text,
        &[
            "--service",
            "cloud-control",
            "--cloud-control",
            "--resource-type",
            "all",
        ],
        &emulator.environment(),
    )?;
    assert_terminal(&report, "universal");
    assert_eq!(
        report.get("status").and_then(Value::as_str),
        Some("complete"),
        "provider boundaries must not make the universal scan partial"
    );

    let block = report
        .pointer("/scans/0/universal")
        .ok_or("universal scan lacks its partition block")?;
    assert_eq!(
        number(block, "total")?,
        EXPECTED_TOTAL,
        "pinned emulator partition total changed"
    );
    assert_eq!(
        number(block, "list_capable")?,
        EXPECTED_LIST_CAPABLE,
        "pinned emulator list-capable count changed"
    );
    assert_eq!(
        number(block, "skipped_parent_required")?,
        EXPECTED_SKIPPED_PARENT_REQUIRED,
        "pinned emulator parent-required skip count changed"
    );
    assert_eq!(
        number(block, "skipped_unsupported")?,
        EXPECTED_SKIPPED_UNSUPPORTED,
        "pinned emulator unsupported-handler skip count changed"
    );
    assert_eq!(
        number(block, "attempted")?,
        EXPECTED_ATTEMPTED,
        "universal scan did not attempt every listable type"
    );
    assert_eq!(
        number(block, "succeeded")?,
        EXPECTED_SUCCEEDED,
        "pinned emulator listed-type success count changed"
    );
    assert_eq!(
        number(block, "unsupported")?,
        EXPECTED_UNSUPPORTED,
        "pinned emulator TypeNotFoundException count changed"
    );
    assert_eq!(
        number(block, "failed")?,
        EXPECTED_FAILED,
        "universal scan recorded a real failure"
    );
    assert_eq!(
        report
            .pointer("/scans/0/summary/edges")
            .and_then(Value::as_u64),
        Some(0),
        "universal mode is list-only: descriptors are built with \
         map_description(..., complete=false), so no GetResource enrichment runs and no static \
         relationship edges are emitted; a non-zero count means enrichment semantics changed and \
         this evidence must be refreshed"
    );
    assert_eq!(
        number(block, "attempted")?,
        number(block, "succeeded")? + number(block, "unsupported")? + number(block, "failed")?,
        "attempted must equal succeeded + unsupported + failed"
    );
    assert_eq!(
        report.pointer("/api/in_flight"),
        Some(&Value::from(0)),
        "universal scan left work in flight"
    );

    let recorded = unsupported_types(&database).await?;
    let evidence = committed_unsupported()?;
    assert_eq!(
        evidence.version, PINNED_VERSION,
        "fixture targets aws-emulator {} but the harness pins {PINNED_VERSION}; \
         a new emulator version requires refreshed evidence",
        evidence.version
    );
    assert_eq!(
        evidence.commit, PINNED_COMMIT,
        "fixture targets commit {} but the harness pins {PINNED_COMMIT}; \
         a new emulator version requires refreshed evidence",
        evidence.commit
    );
    assert_eq!(
        u64::try_from(evidence.types.len())?,
        EXPECTED_UNSUPPORTED,
        "committed fixture does not hold {EXPECTED_UNSUPPORTED} type names"
    );
    assert_eq!(
        recorded, evidence.types,
        "the TypeNotFoundException set differs from committed evidence for aws-emulator \
         {PINNED_VERSION} ({PINNED_COMMIT}); refresh {FIXTURE} and the partition literals \
         only after verifying the delta against the new emulator"
    );
    Ok(())
}

struct Evidence {
    version: String,
    commit: String,
    types: BTreeSet<String>,
}

fn committed_unsupported() -> Result<Evidence, Box<dyn Error>> {
    let path = workspace()?.join(FIXTURE);
    let document: Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    let version = document
        .get("emulator_version")
        .and_then(Value::as_str)
        .ok_or("fixture lacks emulator_version")?
        .to_owned();
    let commit = document
        .get("emulator_commit")
        .and_then(Value::as_str)
        .ok_or("fixture lacks emulator_commit")?
        .to_owned();
    let types = document
        .get("types")
        .and_then(Value::as_array)
        .ok_or("fixture lacks types")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or("fixture type is not a string")
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(Evidence {
        version,
        commit,
        types,
    })
}
