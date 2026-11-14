#[test]
fn catalog_models_reports_embedded_bytes_without_json() -> Result<(), Box<dyn std::error::Error>> {
    assert!(
        !include_bytes!("../catalog/aws-api.catalog").is_empty(),
        "generate catalog first: cargo run -p catalog-tool -- models compile --write"
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
        .args(["catalog", "--models"])
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stderr, b"");
    assert_eq!(
        output.stdout,
        include_bytes!("../catalog/aws-models.report")
    );
    Ok(())
}
