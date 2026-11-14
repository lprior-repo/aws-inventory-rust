use inventory_harness::HarnessError;
use serde_json::{Value, json};

use super::fixture::{Fixture, TestResult, fixture, free, save_summary};
use super::put;

#[tokio::test]
async fn given_current_pins_when_frozen_then_published_provenance_matches_reviewed_release()
-> TestResult {
    let fixture = fixture().await?;

    free(&fixture).await?;
    let frozen: Value = serde_json::from_slice(&std::fs::read(&fixture.destination)?)?;

    assert_eq!(
        frozen
            .pointer("/provenance/emulator_version")
            .and_then(Value::as_str),
        Some("1.0.348")
    );
    assert_eq!(
        frozen
            .pointer("/provenance/emulator_commit")
            .and_then(Value::as_str),
        Some("7461ab1bbe540afe8063ce03ee5477ebb649df48")
    );
    Ok(())
}

#[tokio::test]
async fn given_historical_version_when_frozen_then_invalid_identity_prevents_publication()
-> TestResult {
    let mut fixture = fixture().await?;
    put(
        &mut fixture.summary,
        "/emulator/version_output",
        json!("aws-emulator 1.0.336"),
    )?;
    save_summary(&fixture)?;

    assert_refused(
        &fixture,
        "snapshot baseline emulator identity does not match the pinned fixture",
    )
    .await
}

#[tokio::test]
async fn given_historical_commit_when_frozen_then_invalid_identity_prevents_publication()
-> TestResult {
    let mut fixture = fixture().await?;
    put(
        &mut fixture.summary,
        "/emulator/source_commit",
        json!("f515581569b98a9dcbd2e686c12f912fdce51bba"),
    )?;
    save_summary(&fixture)?;

    assert_refused(
        &fixture,
        "snapshot baseline emulator identity does not match the pinned fixture",
    )
    .await
}

#[tokio::test]
async fn given_absent_version_when_frozen_then_invalid_missing_version_prevents_publication()
-> TestResult {
    let mut fixture = fixture().await?;
    fixture
        .summary
        .get_mut("emulator")
        .and_then(Value::as_object_mut)
        .ok_or("emulator object")?
        .remove("version_output")
        .ok_or("version_output must exist before removal")?;
    save_summary(&fixture)?;

    assert_refused(&fixture, "snapshot baseline lacks emulator version").await
}

async fn assert_refused(fixture: &Fixture, cause: &str) -> TestResult {
    let error = free(fixture)
        .await
        .err()
        .ok_or("invalid evidence must be refused")?;
    assert!(
        matches!(&error, HarnessError::Invalid(reason) if reason == cause),
        "expected Invalid({cause:?}), got {error:?}"
    );
    assert!(!fixture.destination.try_exists()?);
    Ok(())
}
