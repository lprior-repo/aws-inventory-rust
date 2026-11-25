use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn args(work: &Path) -> Args {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Args {
        database: "postgres://localhost/inventory".into(),
        endpoint: "https://localhost".into(),
        ca_bundle: work.join("ca.pem"),
        region: "us-east-1".into(),
        account: None,
        report: work.join("report.tsv"),
        catalog: root.join("catalog/aws-api.catalog"),
        universal: root.join("catalog/cloudcontrol-universal.json"),
        differences: work.join("differences.tsv"),
        seeds: root.join("crates/harness/seeds"),
        oracle: None,
    }
}

fn select(args: &mut Args, kinds: &BTreeSet<String>) -> TestResult {
    use std::fmt::Write;
    let mut oracle = String::new();
    for kind in kinds {
        writeln!(
            oracle,
            "resource\t{kind}\t123456789012\tus-east-1\tidentifier"
        )?;
    }
    let path = args.differences.with_file_name("selected.tsv");
    std::fs::write(&path, oracle)?;
    args.oracle = Some(path);
    Ok(())
}

#[test]
fn given_real_seeds_when_known_input_gaps_are_removed_then_remaining_kinds_are_accepted()
-> TestResult {
    let work = tempfile::tempdir()?;
    let mut args = args(work.path());
    std::fs::write(&args.ca_bundle, "test trust bundle")?;
    std::fs::write(&args.differences, "# type\trule\treason\n")?;
    let error = read(&args).err().ok_or("expected unknown kind error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control kind not in catalog: [\"AWS::ChimeSDKMeetings::Meeting\"]"
    );
    let HarnessError::CloudControl(CloudControlError::UnknownKinds { kinds }) = error else {
        return Err("expected typed unknown kind error".into());
    };
    assert_eq!(kinds, ["AWS::ChimeSDKMeetings::Meeting"]);
    let mut selection = seeded(&args)?;
    assert!(selection.remove("AWS::ChimeSDKMeetings::Meeting"));
    select(&mut args, &selection)?;
    let error = read(&args)
        .err()
        .ok_or("expected unmapped CFN kind error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control seeded kinds have no mapped target: [\"AWS::CloudWatch::Alarm\", \"AWS::ControlTower::LandingZone\", \"AWS::ECS::TaskDefinition\", \"AWS::Glue::Table\"]"
    );
    let HarnessError::CloudControl(CloudControlError::UnmappedKinds { kinds }) = error else {
        return Err("expected typed unmapped CFN kind error".into());
    };
    let gaps = [
        "AWS::CloudWatch::Alarm",
        "AWS::ControlTower::LandingZone",
        "AWS::ECS::TaskDefinition",
        "AWS::Glue::Table",
    ];
    assert_eq!(kinds, gaps);
    for kind in gaps {
        assert!(selection.remove(kind));
    }
    select(&mut args, &selection)?;
    let inputs = read(&args)?;
    let section = super::super::unmapped(&inputs.no_cfn)?;
    assert!(section.contains("AWS::ApiOnly::SSMActivation\tno CloudFormation type\n"));
    assert!(!inputs.seeded.contains("AWS::ApiOnly::SSMActivation"));
    assert!(inputs.seeded.contains("AWS::SNS::Topic"));
    Ok(())
}

#[test]
fn given_seeded_cfn_kinds_without_targets_when_gate_built_then_all_are_sorted_in_typed_error()
-> TestResult {
    let work = tempfile::tempdir()?;
    let args = args(work.path());
    let bytes = std::fs::read(args.catalog)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let error = seed_gate(
        &catalog,
        BTreeSet::from(["AWS::SNS::Topic".to_owned(), "AWS::S3::Bucket".to_owned()]),
        &[],
    )
    .err()
    .ok_or("expected missing target error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control seeded kinds have no mapped target: [\"AWS::S3::Bucket\", \"AWS::SNS::Topic\"]"
    );
    let HarnessError::CloudControl(CloudControlError::UnmappedKinds { kinds }) = error else {
        return Err("expected typed unmapped CFN kind error".into());
    };
    assert_eq!(kinds, ["AWS::S3::Bucket", "AWS::SNS::Topic"]);
    Ok(())
}
