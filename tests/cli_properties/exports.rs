use super::*;

#[test]
fn inventory_export_and_existing_scan_delivery_work_through_actual_cli() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database = path(&directory.path().join("inventory.sqlite"))?;
    let demo = json_command(&arguments(&["demo", "--db", &database]), 0)?;
    let scan = demo
        .get("scan_id")
        .and_then(Value::as_str)
        .context("demo scan ID")?;
    let output = path(&directory.path().join("inventory.json"))?;
    let exported = json_command(
        &arguments(&[
            "export",
            "--db",
            &database,
            "--scan-id",
            scan,
            "--output",
            &output,
            "--format",
            "json",
            "--provenance",
            "all-observations",
        ]),
        0,
    )?;
    assert_eq!(exported.get("resources"), Some(&json!(7)));
    assert_eq!(exported.get("edges"), Some(&json!(4)));
    let document: Value = serde_json::from_slice(&std::fs::read(&output)?)?;
    let records = document
        .get("records")
        .and_then(Value::as_array)
        .context("exported records")?;
    assert_eq!(records.len(), 18);
    assert_eq!(
        document.pointer("/header/artifact"),
        Some(&json!("aws-inventory.snapshot"))
    );
    let original = std::fs::read(&output)?;
    assert!(
        command(
            &arguments(&[
                "export",
                "--db",
                &database,
                "--scan-id",
                scan,
                "--output",
                &output
            ]),
            1
        )?
        .is_empty()
    );
    assert_eq!(std::fs::read(&output)?, original);
    let outputs = path(&directory.path().join("outputs"))?;
    let delivered = json_command(
        &arguments(&[
            "deliver",
            "--db",
            &database,
            "--scan-id",
            scan,
            "--output-dir",
            &outputs,
        ]),
        0,
    )?;
    assert_eq!(delivered.get("delivery_status"), Some(&json!("complete")));
    assert_eq!(
        delivered.pointer("/outputs/inventory/summary/resources"),
        Some(&json!(7))
    );
    let persisted = path(&directory.path().join("jobs.sqlite"))?;
    let submitted = json_command(
        &arguments(&[
            "job",
            "submit",
            "--queue",
            &persisted,
            "--db",
            &database,
            "--output-dir",
            &outputs,
            "--falkor",
            "--job-id",
            "delivery-intent",
        ]),
        0,
    )?;
    assert_eq!(
        submitted.pointer("/job/payload/delivery/falkor"),
        Some(&json!(true))
    );

    assert!(!submitted.to_string().contains("redis://"));
    Ok(())
}
