//! Consumer-visible emulator pins must survive publication, never historical evidence.

use inventory_harness::HarnessError;
use serde_json::{Value, json};

use super::fixture::{Fixture, TestResult, fixture, free, save_summary};
use super::put;

#[tokio::test]
async fn given_current_pins_when_frozen_then_published_provenance_matches_reviewed_release()
-> TestResult {
    // Given: independently asserted synthetic evidence with the current emulator pins.
    let fixture = fixture().await?;
    // When: the real freeze API validates and publishes the oracle.
    free(&fixture).await?;
    let frozen: Value = serde_json::from_slice(&std::fs::read(&fixture.destination)?)?;
    // Then: consumers see the exact reviewed release, not a historical publisher literal.
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
    // Given: only the emulator version in an otherwise valid summary is stale.
    let mut fixture = fixture().await?;
    put(
        &mut fixture.summary,
        "/emulator/version_output",
        json!("aws-emulator 1.0.336"),
    )?;
    save_summary(&fixture)?;
    // When/Then: freeze returns the identity cause and does not publish a destination.
    assert_refused(
        &fixture,
        "snapshot baseline emulator identity does not match the pinned fixture",
    )
    .await
}

#[tokio::test]
async fn given_historical_commit_when_frozen_then_invalid_identity_prevents_publication()
-> TestResult {
    // Given: the current version remains intact; only the source commit is stale.
    let mut fixture = fixture().await?;
    put(
        &mut fixture.summary,
        "/emulator/source_commit",
        json!("f515581569b98a9dcbd2e686c12f912fdce51bba"),
    )?;
    save_summary(&fixture)?;
    // When/Then: freeze rejects the stale commit even alongside the current version.
    assert_refused(
        &fixture,
        "snapshot baseline emulator identity does not match the pinned fixture",
    )
    .await
}

#[tokio::test]
async fn given_absent_version_when_frozen_then_invalid_missing_version_prevents_publication()
-> TestResult {
    // Given: the version_output key is absent, not null or an empty string.
    let mut fixture = fixture().await?;
    fixture
        .summary
        .get_mut("emulator")
        .and_then(Value::as_object_mut)
        .ok_or("emulator object")?
        .remove("version_output")
        .ok_or("version_output must exist before removal")?;
    save_summary(&fixture)?;
    // When/Then: freeze reports the missing version rather than publishing an oracle.
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
