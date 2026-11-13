use std::fs;
use std::path::{Path, PathBuf};

use sdk_operations::inventory::{build, pascal_case, render, verify, write};
use sdk_operations::model::{Inventory, SCHEMA_VERSION};
use sdk_operations::registry::parse_directory_name;
use tempfile::TempDir;

fn create_crate(
    root: &Path,
    name: &str,
    version: &str,
    modules: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = root.join(format!("{name}-{version}"));
    let operations = crate_root.join("src").join("operation");
    fs::create_dir_all(&operations)?;
    fs::write(
        crate_root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n"),
    )?;
    for module in modules {
        fs::write(
            operations.join(format!("{module}.rs")),
            format!("// generated {module}\n"),
        )?;
    }
    Ok(())
}

fn fixture(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    create_crate(
        root,
        "aws-sdk-alpha",
        "1.2.3",
        &["list_things", "get_thing"],
    )?;
    create_crate(root, "aws-sdk-beta", "0.1.0", &["get_beta"])?;
    Ok(())
}

fn lockfile(root: &Path, packages: &[(&str, &str)]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = root.join("Cargo.lock");
    let mut text = String::from("version = 4\n");
    for (name, version) in packages {
        text.push_str(&format!(
            "\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n\
             source = \"registry+https://github.com/rust-lang/crates.io-index\"\n"
        ));
    }
    fs::write(&path, text)?;
    Ok(path)
}

#[test]
fn inventory_schema_counts_and_order() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    fixture(temp.path())?;
    let lock = lockfile(
        temp.path(),
        &[
            ("aws-sdk-alpha", "1.2.3"),
            ("aws-sdk-beta", "0.1.0"),
            ("serde", "1.0.0"),
        ],
    )?;
    let catalogue = build(temp.path(), &lock)?;
    assert_eq!(catalogue.schema_version, SCHEMA_VERSION);
    assert_eq!(catalogue.summary.crates, 2);
    assert_eq!(catalogue.summary.operations, 3);
    assert!(!catalogue.source.registry.ends_with("aws-sdk-alpha-1.2.3"));
    assert!(Path::new(&catalogue.source.registry).is_absolute());

    let names: Vec<&str> = catalogue
        .crates
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    assert_eq!(names, vec!["aws-sdk-alpha", "aws-sdk-beta"]);
    let alpha = catalogue.crates.first().ok_or("missing alpha crate")?;
    assert_eq!(alpha.version, "1.2.3");
    let modules: Vec<&str> = alpha
        .operations
        .iter()
        .map(|operation| operation.module.as_str())
        .collect();
    assert_eq!(modules, vec!["get_thing", "list_things"]);
    let first = alpha.operations.first().ok_or("missing first operation")?;
    assert_eq!(first.api_operation, "GetThing");
    assert_eq!(alpha.sha256.len(), 64);
    assert!(alpha.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert!(
        alpha
            .operations
            .iter()
            .all(|operation| operation.source_sha256.len() == 64)
    );
    Ok(())
}

#[test]
fn render_is_deterministic_and_check_detects_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    fixture(temp.path())?;
    let lock = lockfile(
        temp.path(),
        &[("aws-sdk-alpha", "1.2.3"), ("aws-sdk-beta", "0.1.0")],
    )?;
    let first = build(temp.path(), &lock)?;
    let second = build(temp.path(), &lock)?;
    assert_eq!(first, second);
    let bytes = render(&first)?;
    assert_eq!(bytes, render(&second)?);
    assert!(bytes.ends_with(b"\n"));
    let decoded: Inventory = serde_json::from_slice(&bytes)?;
    assert_eq!(decoded, first);

    let out = temp.path().join("aws-sdk-operations.json");
    write(&out, &bytes)?;
    verify(&out, &bytes)?;
    let mut drifted = bytes.clone();
    let head = drifted.first_mut().ok_or("rendered inventory is empty")?;
    *head = b'[';
    let error = verify(&out, &drifted)
        .err()
        .ok_or("verify accepted drifted bytes")?;
    assert!(
        matches!(error, sdk_operations::Error::StaleOutput { .. }),
        "drift must surface as StaleOutput, got: {error}"
    );
    Ok(())
}

#[test]
fn completeness_gate_names_missing_lock_crate() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    fixture(temp.path())?;
    let lock = lockfile(
        temp.path(),
        &[
            ("aws-sdk-alpha", "1.2.3"),
            ("aws-sdk-beta", "0.1.0"),
            ("aws-sdk-gamma", "9.9.9"),
        ],
    )?;
    let error = build(temp.path(), &lock)
        .err()
        .ok_or("expected a completeness failure")?;
    let message = error.to_string();
    assert!(message.contains("aws-sdk-gamma-9.9.9"), "{message}");
    assert!(message.contains("absent from registry"), "{message}");
    Ok(())
}

#[test]
fn directory_names_parse_without_guessing() {
    assert_eq!(
        parse_directory_name("aws-sdk-cloudcontrol-1.122.0"),
        Some(("aws-sdk-cloudcontrol".to_owned(), "1.122.0".to_owned()))
    );
    assert_eq!(
        parse_directory_name("aws-sdk-marketplace-catalog-2.0.1"),
        Some(("aws-sdk-marketplace-catalog".to_owned(), "2.0.1".to_owned()))
    );
    assert_eq!(parse_directory_name("aws-sdk-cloudcontrol"), None);
    assert_eq!(parse_directory_name("cloudcontrol-1.0.0"), None);
    assert_eq!(
        parse_directory_name("aws-sdk-s3-1.0.0-rc.1"),
        Some(("aws-sdk-s3".to_owned(), "1.0.0-rc.1".to_owned()))
    );
    assert_eq!(pascal_case("list_resources"), "ListResources");
    assert_eq!(pascal_case("get_thing"), "GetThing");
}

#[test]
fn real_registry_covers_every_pinned_sdk_crate() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("/cache/cargo-shared/registry/src");
    let Some(registry) = first_registry(root)? else {
        return Ok(());
    };
    let lockfile = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let catalogue = build(&registry, &lockfile)?;
    assert_eq!(catalogue.schema_version, SCHEMA_VERSION);
    assert!(
        catalogue.summary.crates >= 440,
        "crates: {}",
        catalogue.summary.crates
    );
    assert!(
        catalogue.summary.operations >= 5_000,
        "operations: {}",
        catalogue.summary.operations
    );
    assert!(catalogue.source.registry.contains("index.crates.io-"));
    for pair in catalogue.crates.windows(2) {
        let left = pair.first().ok_or("empty crate pair")?;
        let right = pair.get(1).ok_or("empty crate pair")?;
        assert!((&left.name, &left.version) <= (&right.name, &right.version));
        for operation_pair in left.operations.windows(2) {
            let before = operation_pair.first().ok_or("empty operation pair")?;
            let after = operation_pair.get(1).ok_or("empty operation pair")?;
            assert!(before.module < after.module);
        }
    }
    let bytes = render(&catalogue)?;
    let decoded: Inventory = serde_json::from_slice(&bytes)?;
    assert_eq!(decoded, catalogue);
    Ok(())
}

fn first_registry(root: &Path) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if !root.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("index.crates.io-")
        {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}
