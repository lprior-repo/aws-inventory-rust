use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{collections::BTreeSet, process::Command};

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

fn cargo_output(arguments: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("timeout")
        .args(["120", env!("CARGO")])
        .args(arguments)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()?;
    ensure!(
        output.status.success(),
        "cargo failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    ensure!(
        output.stdout.len() <= 33_554_432,
        "cargo output exceeds 32 MiB"
    );
    Ok(output.stdout)
}

fn metadata(feature: &str) -> Result<Metadata> {
    let bytes = cargo_output(&[
        "metadata",
        "--locked",
        "--offline",
        "--format-version",
        "1",
        "--no-default-features",
        "--features",
        feature,
    ])?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn normal_tree(feature: &str) -> Result<String> {
    let bytes = cargo_output(&[
        "tree",
        "--locked",
        "--offline",
        "-p",
        "aws-inventory",
        "--no-default-features",
        "--features",
        feature,
        "-e",
        "normal",
        "--prefix",
        "none",
        "--format",
        "{p}",
    ])?;
    String::from_utf8(bytes).context("cargo tree is not UTF-8")
}

fn normal_sdk_dependencies(feature: &str) -> Result<BTreeSet<String>> {
    let metadata = metadata(feature)?;
    let tree = normal_tree(feature)?;
    let names: BTreeSet<String> = tree
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| name.starts_with("aws-sdk-"))
        .map(str::to_owned)
        .collect();
    ensure!(
        names.iter().all(|name| metadata
            .packages
            .iter()
            .any(|package| &package.name == name)),
        "normal dependency missing from cargo metadata"
    );
    Ok(names)
}

#[test]
fn given_only_sweep_when_resolved_then_only_profile_loader_sdks_are_present() -> Result<()> {
    let names = normal_sdk_dependencies("sweep")?;
    assert_eq!(
        names,
        BTreeSet::from([
            "aws-sdk-sso".to_owned(),
            "aws-sdk-ssooidc".to_owned(),
            "aws-sdk-sts".to_owned(),
        ])
    );
    Ok(())
}

const ADAPTERS: [&str; 21] = [
    "cloudcontrol",
    "cloudfront",
    "config",
    "dynamodb",
    "ec2",
    "ecs",
    "eks",
    "elasticloadbalancingv2",
    "iam",
    "kms",
    "lambda",
    "organizations",
    "resourceexplorer2",
    "route53",
    "s3",
    "sfn",
    "sns",
    "sqs",
    "sts",
    "sso",
    "ssooidc",
];

#[test]
fn given_aws_when_resolved_then_every_legacy_adapter_remains_available() -> Result<()> {
    let names = normal_sdk_dependencies("aws")?;
    let adapters = ADAPTERS.map(|service| format!("aws-sdk-{service}"));
    assert_eq!(names, BTreeSet::from(adapters));
    Ok(())
}
