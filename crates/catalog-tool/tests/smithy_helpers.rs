pub const MODEL: &str = r#"{"smithy":"2.0","shapes":{
 "n#Svc":{"type":"service","traits":{"aws.protocols#restJson1":{}}},
 "n#ListThings":{"type":"operation","traits":{"smithy.api#readonly":{},"smithy.api#paginated":{}}},
 "n#Input":{"type":"structure","members":{"Shared":{"target":"smithy.api#String"}}}
}}"#;

pub const REPORT: &str = concat!(
    "services 1\noperations 1\nread_named 1\nreadonly 1\nlist_describe 1\n",
    "list_describe_root 1\nservices_with_root 1\npaginated 1\nresources 0\narn_templates 0\n",
    "members 1\ndistinct_member_names 1\njson_services 1\nxml_only_services 0\ncbor_services 0\n",
    "protocol aws.protocols#restJson1 1\n"
);

pub const TWO_REPORT: &str = concat!(
    "services 2\noperations 2\nread_named 2\nreadonly 2\nlist_describe 2\n",
    "list_describe_root 2\nservices_with_root 2\npaginated 2\nresources 0\narn_templates 0\n",
    "members 2\ndistinct_member_names 1\njson_services 2\nxml_only_services 0\ncbor_services 0\n",
    "protocol aws.protocols#restJson1 2\n"
);

pub fn write_fixture(root: &std::path::Path, relative: &str, text: &str) -> std::io::Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)
}

pub fn fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    write_fixture(dir.path(), "models/service/deep/model.json", MODEL)?;
    Ok(dir)
}

pub fn tool(args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(args)
        .output()
}

pub fn report(root: &std::path::Path, flags: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["models", "report", "--root"])
        .arg(root)
        .arg("--source")
        .arg(root)
        .args(flags)
        .output()
}

pub fn success(output: &std::process::Output, expected: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(output.stderr, b"");
}

pub fn success_silent(output: &std::process::Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}
