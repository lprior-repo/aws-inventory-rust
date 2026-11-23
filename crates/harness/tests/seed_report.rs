#[path = "seed_report/server.rs"]
mod server;

use aws_wire::sign::Credentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_harness::inventory_seed::{SeedConfig, SeedRunReport, StepOutcome, load, run_report};
use server::{CALLER, Result, Server};
use std::{path::Path, time::Duration};

const SCENARIO: &str = r#"format = 1
service = "ec2"
[[step]]
id = "accepted"
operation = "CreateVpc"
kind = "AWS::EC2::VPC"
identity = "Vpc.VpcId"
[step.input]
CidrBlock = "10.1.0.0/16"
[[step]]
id = "rejected"
operation = "CreateVpc"
kind = "AWS::EC2::VPC"
identity = "Vpc.VpcId"
[step.input]
CidrBlock = "10.2.0.0/16"
[[step]]
id = "dependent"
operation = "CreateSubnet"
kind = "AWS::EC2::Subnet"
identity = "Subnet.SubnetId"
[step.input]
VpcId = { from = "rejected", path = "Vpc.VpcId" }
CidrBlock = "10.2.1.0/24"
[[step]]
id = "independent"
operation = "CreateVpc"
kind = "AWS::EC2::VPC"
identity = "Vpc.VpcId"
[step.input]
CidrBlock = "10.3.0.0/16"
"#;
const VPC_Z: &str = "<CreateVpcResponse><vpc><vpcId>vpc-z</vpcId></vpc></CreateVpcResponse>";
const VPC_A: &str = "<CreateVpcResponse><vpc><vpcId>vpc-a</vpcId></vpc></CreateVpcResponse>";
const DENIED: &str = "<Response><Errors><Error><Code>UnsupportedOperation</Code><Message>not served</Message></Error></Errors></Response>";

fn catalog_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog")
}

async fn execute() -> Result<SeedRunReport> {
    let server = Server::new().await?;
    let config = SeedConfig::new(&format!("https://{}", server.listener.local_addr()?), &server.ca, "us-east-1")?;
    let bytes = std::fs::read(catalog_path())?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let credentials = Credentials::new("SEEDREPORT", "test-secret", None)?;
    let scenario = load(SCENARIO)?;
    let replies = [(200, CALLER), (200, VPC_Z), (400, DENIED), (200, VPC_A)];
    let (report, requests) = tokio::time::timeout(Duration::from_secs(15), async {
        tokio::join!(run_report(&catalog, &config, &credentials, &scenario), server.serve(&replies))
    }).await?;
    assert_eq!(requests?.len(), 4);
    Ok(report?)
}

#[tokio::test]
async fn given_rejected_parent_when_running_then_dependents_skip_and_sorted_oracle_is_stable() -> Result {
    let first = execute().await?;
    let second = execute().await?;
    assert_eq!(first.oracle_rows(), second.oracle_rows());
    assert_eq!(first.oracle_rows(), vec![
        "resource\tAWS::EC2::VPC\t123456789012\tus-east-1\tvpc-a",
        "resource\tAWS::EC2::VPC\t123456789012\tus-east-1\tvpc-z",
    ]);
    assert!(matches!(first.outcomes.as_slice(), [StepOutcome::Accepted,
        StepOutcome::Rejected { code, .. }, StepOutcome::Skipped { .. }, StepOutcome::Accepted]
        if code == "UnsupportedOperation"));
    let scenario = load(SCENARIO)?;
    let labels = ["accepted", "rejected", "dependent", "independent"].map(String::from);
    let rows = first.matrix_rows(scenario.steps(), &labels)?;
    assert!(rows.iter().any(|row| row.ends_with("skipped: depends on rejected")));
    Ok(())
}

#[tokio::test]
async fn given_no_python_path_when_seed_cli_runs_then_rejections_still_write_both_reports_and_exit_zero() -> Result {
    let server = Server::new().await?;
    let work = tempfile::tempdir()?;
    let seeds = work.path().join("seeds");
    std::fs::create_dir(&seeds)?;
    std::fs::write(seeds.join("ec2.toml"), SCENARIO)?;
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command.args(["seed", "run", "--seeds"]).arg(&seeds).arg("--catalog").arg(catalog_path())
        .arg("--endpoint").arg(format!("https://{}", server.listener.local_addr()?))
        .arg("--ca-bundle").arg(&server.ca).args(["--region", "us-east-1", "--service", "ec2", "--oracle"])
        .arg(work.path().join("oracle")).arg("--matrix").arg(work.path().join("matrix"))
        .env("PATH", "").env("AWS_ACCESS_KEY_ID", "SEEDREPORT").env("AWS_SECRET_ACCESS_KEY", "test-secret").kill_on_drop(true);
    let replies = [(200, CALLER), (200, VPC_Z), (400, DENIED), (200, VPC_A)];
    let (output, requests) = tokio::time::timeout(Duration::from_secs(15), async {
        tokio::join!(command.output(), server.serve(&replies))
    }).await?;
    let output = output?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(requests?.len(), 4);
    assert_sorted(&std::fs::read_to_string(work.path().join("oracle"))?);
    let matrix = std::fs::read_to_string(work.path().join("matrix"))?;
    assert_sorted(&matrix);
    assert!(matrix.contains("rejected\tUnsupportedOperation"));
    Ok(())
}

fn assert_sorted(text: &str) {
    assert!(text.lines().zip(text.lines().skip(1)).all(|(left, right)| left <= right));
}
