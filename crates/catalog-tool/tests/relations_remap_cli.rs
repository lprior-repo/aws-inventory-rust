use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ROW: &str =
    "relation\tAWS::EC2::Subnet\tVpcId\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n";
const REPORT: &str = "relations mapped_live=1 mapped_held=0 unmapped=0 total=1\ntype\tAWS::EC2::Subnet\tmapped_live=1\tmapped_held=0\tunmapped=0\ntype\tAWS::EC2::VPC\tmapped_live=0\tmapped_held=0\tunmapped=0\nmapped, held: path crosses a list (needs E2.2) count=0\n";

fn given_root(json: bool) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let catalog = root.path().join("catalog");
    fs::create_dir(&catalog)?;
    fs::write(
        catalog.join("cloudcontrol-universal.json"),
        include_bytes!("../../../catalog/cloudcontrol-universal.json"),
    )?;
    fs::write(
        catalog.join("aws-api.catalog"),
        include_bytes!("../../../catalog/aws-api.catalog"),
    )?;
    let resources = serde_json::json!({"resource_types":[
        {"type_name":"AWS::EC2::Subnet","kind":"aws_ec2_subnet","view_name":"aws_cc_ec2_subnet",
         "relationships":[{"pointer":"/VpcId","relation":"IN_VPC","target_kind":"aws_ec2_vpc","target_format":"template"}]},
        {"type_name":"AWS::EC2::VPC","kind":"aws_ec2_vpc","view_name":"aws_cc_ec2_vpc"}
    ]});
    let empty = serde_json::json!({"resource_types":[]});
    [
        ("cloudcontrol-resources", resources),
        ("cloudcontrol-parent-resources", empty),
    ]
    .into_iter()
    .try_for_each(|(stem, value)| {
        let (extension, text) = if json {
            ("json", serde_json::to_string(&value)?)
        } else {
            ("toml", toml::to_string(&value)?)
        };
        fs::write(catalog.join(format!("{stem}.{extension}")), text)?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    Ok(root)
}

fn when_remap(root: &Path, mode: &str) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["relations", "remap", "--root"])
        .arg(root)
        .arg("--source")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models"))
        .arg(mode)
        .output()
}

fn then_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn given_json_catalogs_when_written_twice_and_checked_then_both_files_are_identical() -> TestResult
{
    let root = given_root(true)?;
    then_success(&when_remap(root.path(), "--write")?);
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::ROWS_PATH))?,
        ROW
    );
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::REPORT_PATH))?,
        REPORT
    );
    then_success(&when_remap(root.path(), "--write")?);
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::ROWS_PATH))?,
        ROW
    );
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::REPORT_PATH))?,
        REPORT
    );
    then_success(&when_remap(root.path(), "--check")?);
    Ok(())
}

#[test]
fn given_current_toml_catalogs_when_written_then_format_migration_preserves_the_rows() -> TestResult
{
    let root = given_root(false)?;
    then_success(&when_remap(root.path(), "--write")?);
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::ROWS_PATH))?,
        ROW
    );
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::REPORT_PATH))?,
        REPORT
    );
    then_success(&when_remap(root.path(), "--check")?);
    Ok(())
}

#[test]
fn given_list_crossing_rule_when_remapped_then_row_is_reported_held_and_never_loaded_live()
-> TestResult {
    let root = given_root(true)?;
    let path = root.path().join("catalog/cloudcontrol-resources.json");
    let mut input: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    let pointer = input
        .pointer_mut("/resource_types/0/relationships/0/pointer")
        .ok_or("missing rule")?;
    *pointer = serde_json::Value::String("/Ipv6CidrBlockAssociationSet/*/Ipv6CidrBlock".to_owned());
    fs::write(path, serde_json::to_vec(&input)?)?;
    then_success(&when_remap(root.path(), "--write")?);
    assert_eq!(
        fs::read_to_string(root.path().join(catalog_tool::relations::ROWS_PATH))?,
        "relation-held\tAWS::EC2::Subnet\tIpv6CidrBlockAssociationSet.Ipv6CidrBlock\tAWS::EC2::VPC\tVpcId\tIN_VPC\taccount_region\n"
    );
    let report = fs::read_to_string(root.path().join(catalog_tool::relations::REPORT_PATH))?;
    assert_eq!(
        report.lines().next(),
        Some("relations mapped_live=0 mapped_held=1 unmapped=0 total=1")
    );
    assert_eq!(
        report
            .lines()
            .find(|line| line.starts_with("mapped, held:")),
        Some("mapped, held: path crosses a list (needs E2.2) count=1")
    );
    let kinds = catalog_tool::kind_map::load(
        root.path(),
        include_bytes!("../../../catalog/cloudcontrol-universal.json"),
    )?;
    assert_eq!(
        kinds
            .relations()
            .iter()
            .filter(|row| row.source
                == catalog_tool::kind_map::ReviewedRelationSource::SelectedOutput(
                    catalog_tool::kind_map::ReviewedValuePath::Fields(
                        "Ipv6CidrBlockAssociationSet.Ipv6CidrBlock".to_owned()
                    )
                ))
            .count(),
        0
    );
    then_success(&when_remap(root.path(), "--check")?);
    Ok(())
}

#[test]
fn given_stale_rows_or_report_when_checked_then_exact_mismatch_fails_without_rewriting()
-> TestResult {
    let root = given_root(true)?;
    then_success(&when_remap(root.path(), "--write")?);
    for (relative, original) in [
        (catalog_tool::relations::ROWS_PATH, ROW),
        (catalog_tool::relations::REPORT_PATH, REPORT),
    ] {
        let path = root.path().join(relative);
        let stale = format!("{original}x");
        fs::write(&path, &stale)?;
        let output = when_remap(root.path(), "--check")?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        assert_eq!(
            String::from_utf8(output.stderr)?,
            format!(
                "line {} differs: expected None, got Some(\"x\")\n\n",
                original.lines().count().saturating_add(1)
            )
        );
        assert_eq!(fs::read_to_string(&path)?, stale);
        fs::write(path, original)?;
    }
    Ok(())
}

#[test]
fn given_missing_rows_or_report_when_checked_then_exact_missing_file_error_is_returned()
-> TestResult {
    let root = given_root(true)?;
    for relative in [
        catalog_tool::relations::ROWS_PATH,
        catalog_tool::relations::REPORT_PATH,
    ] {
        then_success(&when_remap(root.path(), "--write")?);
        let path = root.path().join(relative);
        fs::remove_file(&path)?;
        let expected = fs::read_to_string(&path)
            .err()
            .ok_or("expected missing file")?;
        let output = when_remap(root.path(), "--check")?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        assert_eq!(
            String::from_utf8(output.stderr)?,
            format!("{}: {expected}\n", path.display())
        );
        assert!(!path.exists());
    }
    Ok(())
}

#[test]
fn given_unwritable_report_path_when_written_then_existing_rows_are_unchanged() -> TestResult {
    let root = given_root(true)?;
    let rows = root.path().join(catalog_tool::relations::ROWS_PATH);
    let report = root.path().join(catalog_tool::relations::REPORT_PATH);
    fs::write(&rows, b"old rows\n")?;
    fs::create_dir(&report)?;
    fs::write(report.join("old-report"), b"old report\n")?;
    let output = when_remap(root.path(), "--write")?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!("{}: Is a directory (os error 21)\n", report.display())
    );
    assert_eq!(fs::read(rows)?, b"old rows\n");
    assert_eq!(fs::read(report.join("old-report"))?, b"old report\n");
    assert_eq!(fs::read_dir(root.path().join("catalog"))?.count(), 6);
    Ok(())
}

#[test]
fn given_same_length_one_byte_change_when_checked_then_rows_and_report_each_fail_without_rewriting()
-> TestResult {
    let root = given_root(true)?;
    then_success(&when_remap(root.path(), "--write")?);
    for (relative, original) in [
        (catalog_tool::relations::ROWS_PATH, ROW),
        (catalog_tool::relations::REPORT_PATH, REPORT),
    ] {
        let path = root.path().join(relative);
        let stale = format!(
            "R{}",
            original.strip_prefix('r').ok_or("expected initial r")?
        );
        assert_eq!(stale.len(), original.len());
        fs::write(&path, &stale)?;
        let output = when_remap(root.path(), "--check")?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"");
        let before = original
            .split_inclusive('\n')
            .next()
            .ok_or("missing expected line")?;
        let after = stale
            .split_inclusive('\n')
            .next()
            .ok_or("missing changed line")?;
        assert_eq!(
            String::from_utf8(output.stderr)?,
            format!(
                "line 1 differs: expected {:?}, got {:?}\n\n",
                Some(before),
                Some(after)
            )
        );
        assert_eq!(fs::read_to_string(&path)?, stale);
        fs::write(path, original)?;
    }
    Ok(())
}
