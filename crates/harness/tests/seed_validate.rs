use std::{path::PathBuf, process::Command};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const VALID: &str = "format=1\nservice='ec2'\n[[step]]\nid='vpc'\noperation='CreateVpc'\nkind='AWS::EC2::VPC'\nidentity='Vpc.VpcId'\n[step.input]\nCidrBlock='10.88.0.0/16'\n";

fn command(directory: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command.args(["seed", "validate", "--seeds"]).arg(directory)
        .arg("--catalog").arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"))
        .env_remove("AWS_ACCESS_KEY_ID").env_remove("AWS_SECRET_ACCESS_KEY")
        .env_remove("AWS_SESSION_TOKEN");
    command
}

#[test]
fn given_valid_recipe_without_credentials_when_validated_then_no_emulator_is_needed() -> TestResult {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("ec2.toml"), VALID)?;
    let output = command(directory.path()).output()?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout)?, format!("{}\tvalid\n", directory.path().join("ec2.toml").display()));
    Ok(())
}

#[test]
fn given_multiple_invalid_recipes_when_validated_then_every_file_is_reported() -> TestResult {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("a.toml"), VALID.replace("Vpc.VpcId", "vpc.VpcId"))?;
    std::fs::write(directory.path().join("b.toml"), VALID.replace("CreateVpc", "InventVpc"))?;
    let output = command(directory.path()).output()?;
    assert!(!output.status.success());
    let text = String::from_utf8(output.stdout)?;
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains("a.toml\tinvalid\t"));
    assert!(text.contains("b.toml\tinvalid\t"));
    assert!(text.contains("modeled field path is missing"));
    assert!(text.contains("modeled operation is missing from catalog"));
    Ok(())
}

#[test]
fn given_service_selection_when_validated_then_unselected_invalid_file_is_not_loaded() -> TestResult {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("ec2.toml"), VALID)?;
    std::fs::write(directory.path().join("other.toml"), "not a recipe")?;
    let output = command(directory.path()).args(["--service", "ec2"]).output()?;
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?.lines().count(), 1);
    Ok(())
}

const DOCUMENT: &str = "format=1\nservice='bedrock-runtime'\n[[step]]\nid='invoke'\noperation='StartAsyncInvoke'\nkind='AWS::Bedrock::AsyncInvoke'\nidentity='invocationArn'\n[step.input]\nmodelId='amazon.nova-reel-v1:0'\nmodelInput={taskType='TEXT_VIDEO',frames=[{enabled=true,ids=[1,2]}]}\noutputDataConfig={s3OutputDataConfig={s3Uri='s3://inventory-seed/output'}}\n";

#[test]
fn given_nested_document_when_validated_offline_then_arbitrary_members_are_supported() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("bedrock-runtime.toml");
    std::fs::write(&path, DOCUMENT)?;
    let output = command(directory.path()).output()?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout)?, format!("{}\tvalid\n", path.display()));
    Ok(())
}

#[test]
fn given_unknown_member_outside_document_when_validated_then_it_remains_invalid() -> TestResult {
    let directory = tempfile::tempdir()?;
    std::fs::write(
        directory.path().join("bedrock-runtime.toml"),
        DOCUMENT.replace("modelId=", "unknownMember="),
    )?;
    let output = command(directory.path()).output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("modeled field path is missing"));
    Ok(())
}
