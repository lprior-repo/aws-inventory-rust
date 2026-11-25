use catalog_tool::overlay::{self, Origin, OverlayClass, ServiceOverlay};
use std::{error::Error, fs, path::Path};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
const HEADER: &str = "format = 1\nreviewed = \"2026-11-15\"\nreviewer = \"generated\"\n";

fn given_file(root: &Path, service: &str, entries: &str) -> Result {
    let directory = root.join("catalog/overlay/generated");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join(format!("{service}.toml")),
        format!("{HEADER}{entries}"),
    )?;
    Ok(())
}

#[test]
fn given_no_generated_directory_when_loaded_then_all_embedded_policies_are_unchanged() -> Result {
    let root = tempfile::tempdir()?;
    let actual = overlay::load(root.path())?;
    assert_eq!(actual, overlay::embedded()?);
    Ok(())
}

#[test]
fn given_conflicting_generated_entries_when_merged_then_embedded_entries_win() -> Result {
    let root = tempfile::tempdir()?;
    given_file(
        root.path(),
        "ec2",
        "[operation.DescribeVpcs]\nclass = \"defer\"\ndefer_reason = \"generated conflict\"\n[operation.DescribeOverlayOnly]\nclass = \"defer\"\ndefer_reason = \"generated only\"\n",
    )?;
    let original = overlay::embedded()?;
    let merged = overlay::load(root.path())?;
    let hand = service(&original, "ec2")?;
    let actual = service(&merged, "ec2")?;
    then_hand_preserved(actual, hand);
    let name = overlay::OperationName::new("DescribeOverlayOnly")?;
    assert_eq!(actual.origin_of(&name), Origin::Generated);
    assert_eq!(
        actual.operations().get(&name).and_then(|entry| entry.class),
        Some(OverlayClass::Defer)
    );
    assert_eq!(actual.reviewer(), hand.reviewer());
    assert_eq!(actual.reviewed(), hand.reviewed());
    Ok(())
}

fn service<'a>(policies: &'a [ServiceOverlay], name: &str) -> Result<&'a ServiceOverlay> {
    policies
        .iter()
        .find(|service| service.service() == name)
        .ok_or_else(|| "missing service".into())
}

fn then_hand_preserved(actual: &ServiceOverlay, hand: &ServiceOverlay) {
    for (name, policy) in hand.operations() {
        assert_eq!(actual.operations().get(name), Some(policy));
        assert_eq!(actual.origin_of(name), Origin::Hand);
    }
}

#[test]
fn given_wrong_reviewer_when_parsed_then_origin_impersonation_is_rejected() {
    assert!(ServiceOverlay::parse("sample", HEADER, Origin::Hand).is_err());
    let hand = HEADER.replace("\"generated\"", "\"human\"");
    assert!(ServiceOverlay::parse("sample", &hand, Origin::Generated).is_err());
    assert!(ServiceOverlay::parse("sample", &hand, Origin::Hand).is_ok());
}

#[test]
fn given_oversized_generated_file_when_loaded_then_bounded_read_fails() -> Result {
    let root = tempfile::tempdir()?;
    given_file(root.path(), "sample", &" ".repeat(overlay::MAX_FILE_BYTES))?;
    then_limit(root.path());
    Ok(())
}

#[test]
fn given_too_many_generated_files_when_loaded_then_directory_limit_fails() -> Result {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("catalog/overlay/generated");
    fs::create_dir_all(&directory)?;
    for index in 0..4097 {
        fs::write(directory.join(format!("service{index}.toml")), HEADER)?;
    }
    then_limit(root.path());
    Ok(())
}

#[test]
fn given_too_many_directory_entries_when_loaded_then_entry_limit_fails() -> Result {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("catalog/overlay/generated");
    fs::create_dir_all(&directory)?;
    for index in 0..8193 {
        fs::write(directory.join(format!("entry{index}")), "")?;
    }
    then_limit(root.path());
    Ok(())
}

fn then_limit(root: &Path) {
    assert!(matches!(
        overlay::load_generated_dir(root),
        Err(overlay::OverlayError::Limit(_))
    ));
}
