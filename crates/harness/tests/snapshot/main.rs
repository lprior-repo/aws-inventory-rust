mod fixture;
mod provenance;

use serde_json::{Value, json};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};

use fixture::{ROLE, TestResult, fixture, free, save_summary, write_json};

fn put(target: &mut Value, pointer: &str, value: Value) -> TestResult {
    let slot = target.pointer_mut(pointer).ok_or("missing document slot")?;
    *slot = value;
    Ok(())
}

#[tokio::test]
async fn given_asserted_baseline_when_frozen_then_only_local_time_is_removed() -> TestResult {
    let fixture = fixture().await?;
    let receipt = free(&fixture).await?;
    let frozen: Value = serde_json::from_str(&std::fs::read_to_string(&fixture.destination)?)?;
    assert_eq!(frozen.get("schema_version"), Some(&json!(1)));
    assert_eq!(
        frozen.get("assume_role_arn").and_then(Value::as_str),
        Some(ROLE)
    );
    let cases = frozen
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("cases")?;
    assert_eq!(cases.len(), 5);
    let native = cases
        .iter()
        .find(|case| case.get("name").and_then(Value::as_str) == Some("native"))
        .ok_or("native")?;
    assert_eq!(
        native.get("resources").ok_or("resources")?,
        &json!([{"id": "native",
            "properties": {"observed_at": "provider-time", "array": [2, 1]}}])
    );
    assert_eq!(
        native.get("edges").ok_or("edges")?,
        &json!([{"from_id": "native", "to_id": "parent", "properties": {"kept": true}}])
    );
    assert_eq!(
        native.pointer("/scopes/0/complete"),
        Some(&Value::Bool(true))
    );
    assert_eq!(
        receipt.get("cases").and_then(Value::as_array).map(Vec::len),
        Some(5)
    );
    assert_eq!(
        receipt
            .get("snapshot_sha256")
            .and_then(Value::as_str)
            .map(str::len),
        Some(64)
    );
    Ok(())
}

#[tokio::test]
async fn given_foreign_binary_when_frozen_then_evidence_is_rejected() -> TestResult {
    let fixture = fixture().await?;
    let other = fixture.root.join("other-binary");
    std::fs::write(&other, b"different collector")?;
    let error = inventory_harness::snapshot::freeze(
        &fixture.evidence,
        &fixture.destination,
        &other,
        &fixture.root,
    )
    .await
    .err()
    .ok_or("foreign binary must be refused")?;
    assert!(error.to_string().contains("binary differs"));
    assert!(!fixture.destination.exists());
    Ok(())
}

#[tokio::test]
async fn given_changed_source_when_frozen_then_prior_assertions_do_not_apply() -> TestResult {
    let fixture = fixture().await?;
    std::fs::write(fixture.root.join("src/main.rs"), "changed collector\n")?;
    let error = free(&fixture)
        .await
        .err()
        .ok_or("changed source must be refused")?;
    assert!(error.to_string().contains("different collector source"));
    Ok(())
}

#[tokio::test]
async fn given_missing_case_binding_when_frozen_then_five_cases_are_required() -> TestResult {
    let mut fixture = fixture().await?;
    if let Some(bindings) = fixture
        .execution
        .get_mut("scan_commands")
        .and_then(Value::as_object_mut)
    {
        bindings.remove("parents");
    }
    if let Some(execution) = fixture.summary.get_mut("execution") {
        *execution = fixture.execution.clone();
    }
    save_summary(&fixture)?;
    let error = free(&fixture)
        .await
        .err()
        .ok_or("missing binding must be refused")?;
    assert!(error.to_string().contains("five collector commands"));
    Ok(())
}

#[tokio::test]
async fn given_extra_persisted_scan_when_frozen_then_database_identity_is_enforced() -> TestResult {
    let fixture = fixture().await?;
    let options = SqliteConnectOptions::new().filename(fixture.evidence.join("inventory.sqlite"));
    let mut connection = SqliteConnection::connect_with(&options).await?;
    sqlx::query("INSERT INTO scans VALUES ('unrelated-scan','complete')")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    let error = free(&fixture)
        .await
        .err()
        .ok_or("extra scan must be refused")?;
    assert!(
        error
            .to_string()
            .contains("exactly the five asserted scan identities")
    );
    Ok(())
}

#[tokio::test]
async fn given_non_boolean_scope_flag_when_frozen_then_row_is_rejected() -> TestResult {
    let fixture = fixture().await?;
    let options = SqliteConnectOptions::new().filename(fixture.evidence.join("inventory.sqlite"));
    let mut connection = SqliteConnection::connect_with(&options).await?;
    sqlx::query("UPDATE scope_checkpoints SET complete=2 WHERE scan_id='synthetic-native'")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    let error = free(&fixture)
        .await
        .err()
        .ok_or("non-boolean flag must be refused")?;
    assert!(error.to_string().contains("non-boolean scope flag"));
    Ok(())
}

#[tokio::test]
async fn given_partial_case_when_frozen_then_failures_and_failed_scopes_are_preserved() -> TestResult
{
    let mut fixture = fixture().await?;
    let command_path = fixture.evidence.join("sources.json");
    let mut command: Value = serde_json::from_str(&std::fs::read_to_string(&command_path)?)?;
    put(&mut command, "/exit_code", json!(2))?;
    put(&mut command, "/result/status", json!("partial"))?;
    put(&mut command, "/result/scans/0/status", json!("partial"))?;
    let sha = write_json(&command_path, &command)?;
    if let Some(binding) = fixture.execution.pointer_mut("/scan_commands/sources") {
        put(binding, "/sha256", json!(sha))?;
    }
    if let Some(execution) = fixture.summary.get_mut("execution") {
        *execution = fixture.execution.clone();
    }
    save_summary(&fixture)?;
    let options = SqliteConnectOptions::new().filename(fixture.evidence.join("inventory.sqlite"));
    let mut connection = SqliteConnection::connect_with(&options).await?;
    sqlx::query("UPDATE scans SET status='partial' WHERE scan_id='synthetic-sources'")
        .execute(&mut connection)
        .await?;
    sqlx::query(
        "UPDATE scope_checkpoints SET complete=0,failed=1 WHERE scan_id='synthetic-sources'",
    )
    .execute(&mut connection)
    .await?;
    sqlx::query("INSERT INTO collection_failures VALUES (?,?,?,?)")
        .bind("synthetic-sources")
        .bind("scope-sources")
        .bind("access_denied")
        .bind("literal expected provider error")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    free(&fixture).await?;
    let frozen: Value = serde_json::from_str(&std::fs::read_to_string(&fixture.destination)?)?;
    let sources = frozen
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("cases")?
        .iter()
        .find(|case| case.get("name").and_then(Value::as_str) == Some("sources"))
        .ok_or("sources")?;
    assert_eq!(
        sources.get("status").and_then(Value::as_str),
        Some("partial")
    );
    assert_eq!(
        sources.get("failures"),
        Some(
            &json!([{"scope_key": "scope-sources", "category": "access_denied",
            "message": "literal expected provider error"}])
        )
    );
    assert_eq!(
        sources.pointer("/scopes/0/failed"),
        Some(&Value::Bool(true))
    );
    Ok(())
}
