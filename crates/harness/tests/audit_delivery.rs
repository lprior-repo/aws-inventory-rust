use inventory_harness::HarnessError;
use inventory_harness::audit::{
    Json, catalogs, dependency_evidence, dumps_default, persisted_reads, registry_file,
    write_rollup,
};
use inventory_harness::provenance::file_sha256;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
type BoxError = Box<dyn std::error::Error>;
type TestResult = Result<(), BoxError>;
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn failure<T>(result: Result<T, HarnessError>) -> Result<String, BoxError> {
    match result {
        Err(error) => Ok(error.to_string()),
        Ok(_) => Err(String::from("expected failure").into()),
    }
}
fn at<'a>(value: &'a Value, path: &str) -> Result<&'a Value, BoxError> {
    value
        .pointer(path)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}
fn at_mut<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Value, BoxError> {
    value
        .pointer_mut(path)
        .ok_or_else(|| BoxError::from(format!("missing {path}")))
}
fn contain(text: &str, needle: &str) -> TestResult {
    assert!(text.contains(needle), "missing {needle:?} in {text:?}");
    Ok(())
}
#[test]
fn frozen_cloud_control_catalogs_are_bound() -> TestResult {
    assert_eq!(catalogs(&workspace())?.catalog_sha256.len(), 64);
    Ok(())
}
#[test]
fn rollup_matches_the_committed_evidence_document() -> TestResult {
    let root = workspace();
    let matrix = Json::parse(&std::fs::read_to_string(
        root.join("catalog/cloudcontrol-capabilities.json"),
    )?)?;
    let directory = tempfile::tempdir()?;
    let rendered = directory.path().join("rollup.md");
    write_rollup(&matrix, &rendered)?;
    assert_eq!(
        std::fs::read(&rendered)?,
        std::fs::read(root.join("docs/CLOUDCONTROL_EVIDENCE.md"))?
    );
    let summary = matrix
        .get("summary")
        .cloned()
        .ok_or_else(|| String::from("summary"))?;
    assert!(dumps_default(&summary.to_value()).starts_with("{\"emulator_list_attempted\": 397,"));
    Ok(())
}
#[test]
fn pinned_operation_modules_are_hashed_or_rejected() -> TestResult {
    let root = workspace();
    let module = "src/operation/list_resources.rs";
    let digest = file_sha256(&registry_file("aws-sdk-cloudcontrol", "1.122.0", module)?)?;
    let provider_digest = file_sha256(&registry_file(
        "aws-config",
        "1.13.2",
        "src/sts/assume_role.rs",
    )?)?;
    let provider = json!({"crate": "aws-config", "version": "1.13.2", "path": "src/sts/assume_role.rs", "sha256": provider_digest});
    let matrix = json!({
        "operations": [{"id": "cc.list", "sdk_crate": "aws-sdk-cloudcontrol", "sdk_version": "1.122.0",
                        "api_operation": "ListResources",
                        "wire_source": {"rust_operation_module": module, "rust_operation_module_sha256": digest}}],
        "credential_provider_boundaries": [{"provider_source": provider}],
    });
    let (rows, pinned) = dependency_evidence(&root, &matrix)?;
    assert_eq!(rows.len(), 1);
    assert_eq!(at(&pinned, "/current_sha256")?, &json!(provider_digest));
    let mut stale = matrix.clone();
    *at_mut(
        &mut stale,
        "/operations/0/wire_source/rust_operation_module_sha256",
    )? = json!("0".repeat(64));
    assert_eq!(
        failure(dependency_evidence(&root, &stale))?,
        "pinned SDK operation module changed: cc.list"
    );
    Ok(())
}
#[tokio::test]
async fn persisted_reads_are_scoped_and_schema_bound() -> TestResult {
    use sqlx::Connection;
    use sqlx::sqlite::SqliteConnectOptions;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let mut connection = sqlx::SqliteConnection::connect_with(&options).await?;
    sqlx::query("CREATE TABLE scans(scan_id TEXT,status TEXT)")
        .execute(&mut connection)
        .await?;
    sqlx::query("CREATE TABLE resource_observations(scan_id TEXT,source TEXT,resource_json TEXT)")
        .execute(&mut connection)
        .await?;
    sqlx::query("INSERT INTO scans VALUES ('scan','partial')")
        .execute(&mut connection)
        .await?;
    let resource = json!({"source": "cloud_control", "properties": {
        "cloudformation_type": "AWS::Example::Thing", "schema_sha256": "a".repeat(64),
        "details_available": true, "observation_detail": "get_resource"}})
    .to_string();
    for (scan, source) in [
        ("scan", "cloud_control"),
        ("other", "cloud_control"),
        ("scan", "direct"),
    ] {
        sqlx::query("INSERT INTO resource_observations VALUES (?,?,?)")
            .bind(scan)
            .bind(source)
            .bind(&resource)
            .execute(&mut connection)
            .await?;
    }
    drop(connection);
    let catalog = Json::from_value(&json!({"resource_types": [
        {"type_name": "AWS::Example::Thing", "schema_sha256": "a".repeat(64)}]}));
    let counts = persisted_reads(&path, "scan", &catalog, None).await?;
    assert_eq!(counts.get("AWS::Example::Thing"), Some(&1));
    contain(
        &failure(persisted_reads(&path, "scan", &catalog, Some("complete")).await)?,
        "status differs",
    )?;
    contain(
        &failure(persisted_reads(&path, "other", &catalog, None).await)?,
        "missing or not terminal",
    )?;
    let changed = Json::from_value(&json!({"resource_types": [
        {"type_name": "AWS::Example::Thing", "schema_sha256": "b".repeat(64)}]}));
    contain(
        &failure(persisted_reads(&path, "scan", &changed, None).await)?,
        "different provider schema",
    )
}
