use super::{REGION, TestResult, runtime};
use runtime::{NATIVE_SLOT, NativeRuntime};
use std::{path::Path, process::Command};

#[tokio::test(flavor = "current_thread")]
async fn given_sns_model_when_cli_draft_is_run_then_create_topic_is_accepted() -> TestResult {
    // Given an owned emulator and a draft directory outside checked-in seeds.
    let _permit = NATIVE_SLOT.acquire().await?;
    let native = NativeRuntime::start()?;
    let seeds = native.work().join("drafts");
    std::fs::create_dir(&seeds)?;
    let output = seeds.join("sns.toml");
    let draft = Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
        .args(["seed", "draft", "--service", "sns", "--output"])
        .arg(&output)
        .output()?;
    assert_eq!(
        draft.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&draft.stderr)
    );
    // When the exact generated file is passed through the seed run command.
    let matrix = native.work().join("draft-matrix.tsv");
    let result = seed_run(&native, &seeds, &matrix)?;
    // Then the generated topic step has an accepted matrix entry.
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let rows = std::fs::read_to_string(matrix)?;
    assert_eq!(
        rows.lines()
            .find(|line| line.starts_with("sns\tCreateTopic\t")),
        Some("sns\tCreateTopic\tcreatetopic\taccepted")
    );
    native.stop()
}

fn seed_run(
    native: &NativeRuntime,
    seeds: &Path,
    matrix: &Path,
) -> TestResult<std::process::Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    command
        .args(["seed", "run", "--seeds"])
        .arg(seeds)
        .arg("--catalog")
        .arg(runtime::catalog_path())
        .args(["--endpoint", native.emulator().endpoint(), "--ca-bundle"])
        .arg(native.emulator().ca_bundle())
        .args(["--region", REGION, "--service", "sns", "--oracle"])
        .arg(native.work().join("draft-oracle.tsv"))
        .arg("--matrix")
        .arg(matrix);
    std::fs::read_to_string(native.emulator().credentials())?
        .lines()
        .filter_map(|line| line.split_once('='))
        .try_for_each(|(name, value)| {
            let variable = match name.trim() {
                "aws_access_key_id" => "AWS_ACCESS_KEY_ID",
                "aws_secret_access_key" => "AWS_SECRET_ACCESS_KEY",
                "aws_session_token" => "AWS_SESSION_TOKEN",
                _ => return Ok::<_, Box<dyn std::error::Error>>(()),
            };
            command.env(variable, value.trim());
            Ok(())
        })?;
    Ok(command.output()?)
}
