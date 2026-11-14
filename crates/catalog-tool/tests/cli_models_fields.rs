include!("smithy_helpers.rs");

fn fields(root: &std::path::Path, flags: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["models", "fields", "--root"])
        .arg(root)
        .arg("--source")
        .arg(root)
        .args(flags)
        .output()
}

fn registry_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join("catalog/field-ids.txt")
}

fn read_registry(root: &std::path::Path) -> std::io::Result<String> {
    std::fs::read_to_string(registry_path(root))
}

const MODEL_A: &str = r#"{"shapes":{
"n#S":{"type":"structure","members":{"Name":{},"Age":{}}},
"n#U":{"type":"union","members":{"Name":{},"Email":{}}}}}"#;
const MODEL_B: &str = r#"{"shapes":{
"n#S":{"type":"structure","members":{"Email":{}}}}}"#;
const ADDED: &str = r#"{"shapes":{
"n#S":{"type":"structure","members":{"Name":{},"Age":{},"Aardvark":{}}},
"n#U":{"type":"union","members":{"Name":{},"Email":{}}}}}"#;

fn field_fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    write_fixture(root.path(), "models/a.json", MODEL_A)?;
    Ok(root)
}

fn failure(output: &std::process::Output, message: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(String::from_utf8_lossy(&output.stderr), message);
}

#[test]
fn bootstrap_deduplicates_structure_union_and_reversed_models()
-> Result<(), Box<dyn std::error::Error>> {
    // Given two model trees with opposite sorted discovery order.
    let first = field_fixture()?;
    write_fixture(first.path(), "models/z.json", MODEL_B)?;
    let second = tempfile::tempdir()?;
    write_fixture(second.path(), "models/a.json", MODEL_B)?;
    write_fixture(second.path(), "models/z.json", MODEL_A)?;
    // When bootstrapping, then both trees have the independently specified bytes.
    success(&fields(first.path(), &["--write"])?, "field-ids 3 (+3)\n");
    success(&fields(second.path(), &["--write"])?, "field-ids 3 (+3)\n");
    assert_eq!(read_registry(first.path())?, "Age\nEmail\nName\n");
    assert_eq!(read_registry(second.path())?, "Age\nEmail\nName\n");
    Ok(())
}

#[test]
fn second_write_preserves_bytes_and_modification_time() -> Result<(), Box<dyn std::error::Error>> {
    // Given an existing registry with a deliberately old modification time.
    let root = field_fixture()?;
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+3)\n");
    let file = std::fs::File::open(registry_path(root.path()))?;
    let old = std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(1))
        .ok_or("time")?;
    file.set_times(std::fs::FileTimes::new().set_modified(old))?;
    let before = file.metadata()?.modified()?;
    // When regenerated, then unchanged content is not rewritten.
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+0)\n");
    assert_eq!(read_registry(root.path())?, "Age\nEmail\nName\n");
    assert_eq!(
        std::fs::metadata(registry_path(root.path()))?.modified()?,
        before
    );
    success(&fields(root.path(), &["--check"])?, "field-ids 3 (+0)\n");
    Ok(())
}

#[test]
fn check_names_new_member_without_writing() -> Result<(), Box<dyn std::error::Error>> {
    // Given a new member that sorts before every existing member.
    let root = field_fixture()?;
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+3)\n");
    write_fixture(root.path(), "models/a.json", ADDED)?;
    // When checking, then the exact diagnostic names it and preserves the registry.
    failure(
        &fields(root.path(), &["--check"])?,
        concat!(
            "1 member names are not in the registry, first: Aardvark; ",
            "run `catalog-tool models fields --write`\n"
        ),
    );
    assert_eq!(read_registry(root.path())?, "Age\nEmail\nName\n");
    assert!(!root.path().join("catalog/field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn write_appends_earlier_name_and_subsequent_writes_preserve_history()
-> Result<(), Box<dyn std::error::Error>> {
    // Given a registry and a lexically earlier new name.
    let root = field_fixture()?;
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+3)\n");
    write_fixture(root.path(), "models/a.json", ADDED)?;
    // When writing twice and checking, then IDs never reorder.
    success(&fields(root.path(), &["--write"])?, "field-ids 4 (+1)\n");
    assert_eq!(read_registry(root.path())?, "Age\nEmail\nName\nAardvark\n");
    success(&fields(root.path(), &["--write"])?, "field-ids 4 (+0)\n");
    success(&fields(root.path(), &["--check"])?, "field-ids 4 (+0)\n");
    Ok(())
}

#[test]
fn removing_member_retains_its_line_and_check_passes() -> Result<(), Box<dyn std::error::Error>> {
    // Given a previously generated member that is retired in the next valid model.
    let root = field_fixture()?;
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+3)\n");
    write_fixture(root.path(), "models/a.json", MODEL_B)?;
    // When writing and checking, then retirement neither removes nor reuses IDs.
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+0)\n");
    assert_eq!(read_registry(root.path())?, "Age\nEmail\nName\n");
    success(&fields(root.path(), &["--check"])?, "field-ids 3 (+0)\n");
    Ok(())
}

fn rejects_corruption(text: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = field_fixture()?;
    write_fixture(root.path(), "catalog/field-ids.txt", text)?;
    ["--write", "--check"].into_iter().try_for_each(|mode| {
        failure(&fields(root.path(), &[mode])?, message);
        assert_eq!(std::fs::read(registry_path(root.path()))?, text.as_bytes());
        assert!(!root.path().join("catalog/field-ids.txt.tmp").exists());
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn malformed_registry_never_changes_on_write_or_check() -> Result<(), Box<dyn std::error::Error>> {
    // Given corrupt registries, when either mode runs, then exact errors and bytes survive.
    rejects_corruption(
        "Age\nAge\n",
        "duplicate member name on registry line 2: \"Age\"\n",
    )?;
    rejects_corruption("Age\nName", "registry is missing its final newline\n")?;
    rejects_corruption("Age\r\n", "carriage return on registry line 1\n")?;
    rejects_corruption(
        "bad-name\n",
        "invalid member name on registry line 1: \"bad-name\"\n",
    )?;
    rejects_corruption("Age\n\n", "invalid member name on registry line 2: \"\"\n")?;
    Ok(())
}

fn malformed_model_error(root: &std::path::Path) -> String {
    format!(
        "invalid JSON in {}: EOF while parsing an object at line 1 column 1\n",
        root.join("models/z-last.json").display()
    )
}

#[test]
fn malformed_later_model_creates_no_directory() -> Result<(), Box<dyn std::error::Error>> {
    // Given a valid first model and malformed second model, with no output tree.
    let root = field_fixture()?;
    write_fixture(root.path(), "models/z-last.json", "{")?;
    // When writing, then failure happens before any directory or partial stdout exists.
    failure(
        &fields(root.path(), &["--write"])?,
        &malformed_model_error(root.path()),
    );
    assert!(!root.path().join("catalog").exists());
    Ok(())
}

#[test]
fn malformed_later_model_preserves_existing_bytes() -> Result<(), Box<dyn std::error::Error>> {
    // Given history and a malformed model after a valid one.
    let root = field_fixture()?;
    write_fixture(root.path(), "catalog/field-ids.txt", "retired\n")?;
    write_fixture(root.path(), "models/z-last.json", "{")?;
    // When writing, then even a valid earlier model cannot partially publish.
    failure(
        &fields(root.path(), &["--write"])?,
        &malformed_model_error(root.path()),
    );
    assert_eq!(std::fs::read(registry_path(root.path()))?, b"retired\n");
    assert!(!root.path().join("catalog/field-ids.txt.tmp").exists());
    Ok(())
}

#[test]
fn missing_registry_is_io_failure_not_staleness() -> Result<(), Box<dyn std::error::Error>> {
    // Given models but no registry, when checking, then the missing file is named.
    let root = field_fixture()?;
    let message = format!(
        "{}: No such file or directory (os error 2)\n",
        registry_path(root.path()).display()
    );
    failure(&fields(root.path(), &["--check"])?, &message);
    assert!(!root.path().join("catalog").exists());
    Ok(())
}

#[test]
fn conflicting_and_unknown_flags_fail_with_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Given invalid syntax, when invoked, then usage fails before model I/O.
    let root = field_fixture()?;
    let cases = [
        (
            &["--write", "--check"][..],
            "repeated or conflicting flag --check",
        ),
        (&["--bogus"][..], "unexpected '--bogus'"),
    ];
    cases.into_iter().try_for_each(|(flags, message)| {
        let output = fields(root.path(), flags)?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        assert!(String::from_utf8(output.stderr)?.starts_with(&format!("{message}; usage:\n")));
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    assert!(!root.path().join("catalog").exists());
    Ok(())
}

#[test]
fn missing_or_unknown_models_subcommand_fails_with_usage() -> Result<(), Box<dyn std::error::Error>>
{
    // Given missing and unknown subcommands, when invoked, then neither is silently accepted.
    let cases = [
        (&["models"][..], "expected 'report' subcommand"),
        (&["models", "unknown"][..], "unexpected 'unknown'"),
    ];
    cases.into_iter().try_for_each(|(args, message)| {
        let output = tool(args)?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        assert!(String::from_utf8(output.stderr)?.starts_with(&format!("{message}; usage:\n")));
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn print_bootstraps_full_candidate_without_creating_files() -> Result<(), Box<dyn std::error::Error>>
{
    // Given no registry, when printing, then exact candidate bytes go only to stdout.
    let root = field_fixture()?;
    success(&fields(root.path(), &[])?, "Age\nEmail\nName\n");
    assert!(!root.path().join("catalog").exists());
    Ok(())
}

#[test]
fn print_extends_existing_history_without_mutating_it() -> Result<(), Box<dyn std::error::Error>> {
    // Given nonalphabetical history including a retired name.
    let root = field_fixture()?;
    write_fixture(root.path(), "catalog/field-ids.txt", "Name\nRetired\nAge\n")?;
    // When printing, then it shows the append-only candidate, not a sorted replacement.
    success(&fields(root.path(), &[])?, "Name\nRetired\nAge\nEmail\n");
    assert_eq!(read_registry(root.path())?, "Name\nRetired\nAge\n");
    Ok(())
}

#[test]
fn generated_lines_have_exact_bidirectional_ids() -> Result<(), Box<dyn std::error::Error>> {
    // Given generated output, when consumed through the public typed API, then IDs are exact.
    let root = field_fixture()?;
    success(&fields(root.path(), &["--write"])?, "field-ids 3 (+3)\n");
    let registry = catalog_tool::smithy::Registry::parse(&read_registry(root.path())?)?;
    let first = inventory_core::record::FieldId::new(1)?;
    let last = inventory_core::record::FieldId::new(3)?;
    assert_eq!(registry.id("Age"), Some(first));
    assert_eq!(registry.id("Name"), Some(last));
    assert_eq!(registry.name(first), Some("Age"));
    assert_eq!(registry.name(last), Some("Name"));
    Ok(())
}
