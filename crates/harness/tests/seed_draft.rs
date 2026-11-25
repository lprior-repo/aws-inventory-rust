use inventory_harness::inventory_seed::load;
use std::process::Command;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_ec2_outside_thin_slice_when_cli_drafts_to_stdout_then_vpc_recipe_parses() -> TestResult {
    // Given the committed compiled catalog and a service outside A4.3's thin slice.
    let command = env!("CARGO_BIN_EXE_inventory-harness");
    // When no --output path is supplied, stdout contains the seed file itself.
    let output = Command::new(command)
        .args(["seed", "draft", "--service", "ec2"])
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stderr, b"");
    let text = String::from_utf8(output.stdout)?;
    let scenario = load(&text)?;
    // Then an actual create step carries the enumerated VPC kind.
    let vpc = scenario
        .steps()
        .iter()
        .find(|step| step.operation == "CreateVpc")
        .ok_or("EC2 draft has no VPC create step")?;
    assert_eq!(vpc.service, "ec2");
    assert_eq!(vpc.kind, "AWS::EC2::VPC");
    Ok(())
}

#[test]
fn given_missing_service_when_cli_drafts_then_failure_is_reported_without_seed_output() -> TestResult
{
    // Given a service selector absent from the committed catalog.
    let command = env!("CARGO_BIN_EXE_inventory-harness");
    // When the draft command resolves that selector through RuntimeCatalog.
    let output = Command::new(command)
        .args(["seed", "draft", "--service", "no-such-inventory-service"])
        .output()?;
    // Then the failure is exact and stdout remains an uncontaminated seed stream.
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "invalid emulator seed configuration: draft service is missing from catalog\n"
    );
    Ok(())
}
