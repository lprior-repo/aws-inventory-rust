use std::process::Command;

#[test]
fn rebuild_graph_rejects_missing_database_before_contacting_the_graph() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
        .args([
            "rebuild-graph",
            "--database",
            "/nonexistent/inventory.db",
            "--graph-url",
            "redis://127.0.0.1:1",
            "--graph-name",
            "inventory",
        ])
        .output()?;
    assert!(!output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("database does not exist"), "{text}");
    Ok(())
}
