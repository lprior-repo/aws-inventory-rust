use catalog_tool::overlay::{self, CheckError, ItemSelector, ViolationRule};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

fn fixture() -> Result<TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("catalog/overlay"))?;
    fs::create_dir_all(root.path().join("source/models/things/service/2020-01-01"))?;
    fs::write(root.path().join("catalog/overlay/must-deny.txt"), "")?;
    fs::write(
        root.path()
            .join("source/models/things/service/2020-01-01/things.json"),
        MODEL,
    )?;
    Ok(root)
}

fn source(root: &Path) -> std::path::PathBuf {
    root.join("source")
}

fn command(root: &Path, mode: &str) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["overlay", "--root"])
        .arg(root)
        .args(["--source"])
        .arg(source(root))
        .arg(mode)
        .output()
}
#[test]
fn item_selector_accepts_explicit_root_without_changing_member_paths() {
    assert_eq!(
        ItemSelector::parse("Envelope.Items"),
        overlay::MemberPath::new("Envelope.Items").map(ItemSelector::Member)
    );
    assert_eq!(ItemSelector::parse("$"), Ok(ItemSelector::Root));
    assert!(ItemSelector::parse("$.").is_err());
}

#[test]
fn empty_overlay_check_builds_a_non_callable_baseline() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    assert!(checked.services().is_empty());
    assert_eq!(
        checked.report(),
        "totals\tservices_with_overlay=0\tcallable_enumerate=0\tcallable_expand=0\tdenied_by_overlay=0\tdeferred=0\tservices_without_overlay=1\tunreviewed_enumerate=1\tunreviewed_expand=0\tunreviewed_unclassified=0\n"
    );
    Ok(())
}

#[test]
fn successful_cli_prints_report_and_drift_preserves_report_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let report = root.path().join("catalog/aws-overlay.report");
    let expected = overlay::build_report(root.path(), &source(root.path()))?;
    let written = command(root.path(), "--write")?;
    assert!(written.status.success());
    assert_eq!(written.stdout, expected.as_bytes());
    assert_eq!(fs::read(&report)?, expected.as_bytes());
    let matching = command(root.path(), "--check")?;
    assert!(matching.status.success());
    assert_eq!(matching.stdout, expected.as_bytes());
    let report_len = expected.len();
    let mut drift = expected.into_bytes();
    assert_eq!(drift.len(), report_len);
    let byte = drift.first_mut().ok_or("empty report")?;
    *byte = if *byte == b't' { b'u' } else { b't' };
    fs::write(&report, &drift)?;
    let output = command(root.path(), "--check")?;
    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(fs::read(report)?, drift);
    Ok(())
}

#[test]
fn reviewed_list_overlay_exposes_only_readonly_callable_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    fs::write(
        root.path().join("catalog/overlay/things.toml"),
        "format=1\nreviewed='2026-11-13'\nreviewer='test'\n[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n",
    )?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    let service = checked
        .services()
        .first()
        .ok_or("missing reviewed service")?;
    let callable = service
        .callable()
        .first()
        .ok_or("missing callable operation")?;
    assert_eq!(callable.operation(), "ListThings");
    assert_eq!(callable.class(), overlay::EffectiveClass::Enumerate);
    assert_eq!(
        callable.plan().items,
        Some(ItemSelector::Member(overlay::MemberPath::new("Items")?))
    );
    assert_eq!(callable.plan().paginator, Some(overlay::Paginator::None));
    assert_eq!(
        callable.plan().identity,
        Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
            "Id"
        )?]))
    );
    assert_eq!(
        checked.report(),
        "totals\tservices_with_overlay=1\tcallable_enumerate=1\tcallable_expand=0\tdenied_by_overlay=0\tdeferred=0\tservices_without_overlay=0\tunreviewed_enumerate=0\tunreviewed_expand=0\tunreviewed_unclassified=0\noperation\tthings\tListThings\tenumerate\tclass=overlay,items=overlay,page_size=missing,paginator=overlay,identity=overlay,bindings=missing,scope=missing,batch=missing\n"
    );
    Ok(())
}

#[test]
fn failed_cli_write_keeps_report_and_emits_no_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let report = root.path().join("catalog/aws-overlay.report");
    fs::write(&report, "prior\n")?;
    fs::write(
        root.path().join("catalog/overlay/things.toml"),
        "format=1\nreviewed='2026-11-13'\nreviewer='test'\n[operation.ListThings]\nclass='enumerate'\n",
    )?;
    let output = command(root.path(), "--write")?;
    assert!(!output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(fs::read(report)?, b"prior\n");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("identity=item selection unavailable"));
    assert!(stderr.contains("items=missing"));
    assert!(stderr.contains("paginator=missing explicit none"));
    Ok(())
}

#[test]
fn root_item_selector_uses_output_structure_not_item_structure()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture()?;
    let model_path = root
        .path()
        .join("source/models/things/service/2020-01-01/things.json");
    let model = MODEL
        .replace(
            "\"Id\":{\"target\":\"smithy.api#String\"},\"Items\"",
            "\"RootId\":{\"target\":\"smithy.api#String\"},\"Items\"",
        )
        .replace(
            "\"n#Thing\":{\"type\":\"structure\",\"members\":{\"Id\"",
            "\"n#Thing\":{\"type\":\"structure\",\"members\":{\"ItemId\"",
        );
    fs::write(model_path, model)?;
    fs::write(
        root.path().join("catalog/overlay/things.toml"),
        "format=1\nreviewed='2026-11-13'\nreviewer='test'\n[operation.ListThings]\nclass='enumerate'\nitems='$'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['RootId']\n",
    )?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    let callable = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .first()
        .ok_or("missing callable")?;
    assert_eq!(callable.plan().items, Some(ItemSelector::Root));
    assert_eq!(
        callable.plan().identity,
        Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
            "RootId"
        )?]))
    );
    fs::write(
        root.path().join("catalog/overlay/things.toml"),
        "format=1\nreviewed='2026-11-13'\nreviewer='test'\n[operation.ListThings]\nclass='enumerate'\nitems='$'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['ItemId']\n",
    )?;
    let errors = match overlay::check(root.path(), &source(root.path())) {
        Err(CheckError::Violations(errors)) => errors,
        Err(error) => return Err(error.into()),
        Ok(_) => return Err("root selector accepted item-only identity".into()),
    };
    assert_eq!(
        errors
            .iter()
            .map(|error| (error.key(), error.value(), error.rule()))
            .collect::<Vec<_>>(),
        [("identity", "ItemId", ViolationRule::InvalidPath)]
    );
    Ok(())
}
