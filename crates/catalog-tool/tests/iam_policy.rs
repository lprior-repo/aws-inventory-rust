use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use catalog_tool::{
    ReportMode,
    iam_policy::{Error as PolicyError, UnmappedPolicy, render, run},
};
use serde_json::Value;

#[path = "iam_policy/document.rs"]
mod document;

type TestResult = Result<(), Box<dyn Error>>;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reviewed_catalog() -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(
        repo_root().join("catalog/aws-operations.toml"),
    )?)
}

fn catalog(operations: &[(&str, Vec<&str>)]) -> String {
    operations
        .iter()
        .map(|(id, actions)| {
            let actions: Vec<String> = actions.iter().map(|action| format!("{action:?}")).collect();
            format!(
                "[[operations]]\nid = {id:?}\niam_actions = [{}]\n",
                actions.join(", ")
            )
        })
        .collect()
}

fn policy_actions(text: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let document: Value = serde_json::from_str(text)?;
    let actions = document
        .pointer("/Statement/0/Action")
        .and_then(Value::as_array)
        .ok_or("Action is not an array")?;
    Ok(actions
        .iter()
        .map(|action| action.as_str().unwrap_or_default().to_owned())
        .collect())
}

#[test]
fn list_buckets_maps_to_its_reviewed_iam_action_not_the_sdk_name() -> TestResult {
    // Given the pinned operation catalog.
    let text = render(&reviewed_catalog()?, UnmappedPolicy::Refuse)?.text;
    // Then the reviewed IAM action is present and the SDK-derived name is not.
    let actions = policy_actions(&text)?;
    assert!(actions.contains(&"s3:ListAllMyBuckets".to_owned()));
    assert!(!actions.contains(&"s3:ListBuckets".to_owned()));
    Ok(())
}

#[test]
fn policy_actions_are_exactly_the_reviewed_operation_actions() -> TestResult {
    // Given every callable operation's reviewed actions, read from the catalog itself.
    let source = reviewed_catalog()?;
    let document: Value = toml::from_str(&source)?;
    let expected: BTreeSet<String> = document
        .get("operations")
        .and_then(Value::as_array)
        .ok_or("operations is not an array")?
        .iter()
        .flat_map(|operation| {
            operation
                .get("iam_actions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|action| action.as_str().map(str::to_owned))
        })
        .collect();
    // When the policy is rendered.
    let actions = policy_actions(&render(&source, UnmappedPolicy::Refuse)?.text)?;
    // Then it grants exactly those actions, and never the credential-minting boundary.
    assert_eq!(actions.iter().cloned().collect::<BTreeSet<_>>(), expected);
    assert_eq!(actions.len(), expected.len());
    assert!(!actions.contains(&"sts:AssumeRole".to_owned()));
    Ok(())
}

#[test]
fn multi_action_operation_contributes_every_action_sorted_and_deduplicated() -> TestResult {
    // Given two operations whose actions overlap and are unsorted.
    let source = catalog(&[
        (
            "ec2.describe_things",
            vec!["ec2:DescribeVpcs", "ec2:DescribeSubnets"],
        ),
        (
            "ec2.describe_more",
            vec!["ec2:DescribeSubnets", "ec2:DescribeRouteTables"],
        ),
    ]);
    // When rendered.
    let actions = policy_actions(&render(&source, UnmappedPolicy::Refuse)?.text)?;
    // Then the policy lists each action once, in byte order.
    assert_eq!(
        actions,
        vec![
            "ec2:DescribeRouteTables",
            "ec2:DescribeSubnets",
            "ec2:DescribeVpcs",
        ]
    );
    Ok(())
}

#[test]
fn unmapped_operations_fail_closed_and_every_one_is_listed() -> TestResult {
    // Given one operation with an empty mapping and one with no mapping field at all.
    let source = format!(
        "{}[[operations]]\nid = \"s3.missing\"\n",
        catalog(&[
            ("s3.list_buckets", vec!["s3:ListAllMyBuckets"]),
            ("s3.empty", vec![]),
        ])
    );
    // When rendered in the default refusing mode.
    let error = render(&source, UnmappedPolicy::Refuse)
        .err()
        .ok_or("unmapped operations must be refused")?;
    // Then no partial policy is produced and both ids are named.
    match error {
        PolicyError::Unmapped { ids } => assert_eq!(ids, vec!["s3.empty", "s3.missing"]),
        other => return Err(format!("unexpected error: {other}").into()),
    }
    Ok(())
}

#[test]
fn report_mode_lists_unmapped_operations_and_excludes_them_from_the_policy() -> TestResult {
    let source = catalog(&[
        ("s3.list_buckets", vec!["s3:ListAllMyBuckets"]),
        ("s3.empty", vec![]),
    ]);
    let rendered = render(&source, UnmappedPolicy::Report)?;
    assert_eq!(rendered.unmapped, vec!["s3.empty"]);
    assert_eq!(policy_actions(&rendered.text)?, vec!["s3:ListAllMyBuckets"]);
    Ok(())
}

#[test]
fn report_mode_cannot_publish_a_partial_policy() -> TestResult {
    // Given a catalog with an unmapped operation in a scratch repository.
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("catalog"))?;
    fs::write(
        dir.path().join("catalog/aws-operations.toml"),
        "[[operations]]\nid = \"s3.list_buckets\"\niam_actions = [\"s3:ListAllMyBuckets\"]\n\n[[operations]]\nid = \"s3.empty\"\niam_actions = []\n",
    )?;
    // When report mode is asked to write or check the policy.
    let written = run(dir.path(), ReportMode::Write, UnmappedPolicy::Report);
    // Then it is refused and nothing is created.
    assert!(matches!(written, Err(PolicyError::ReportOnlyPublication)));
    assert!(
        !dir.path()
            .join("catalog/iam-read-only-policy.json")
            .exists()
    );
    Ok(())
}

#[test]
fn write_actions_are_refused() -> TestResult {
    for action in [
        "s3:DeleteBucket",
        "s3:PutObject",
        "ec2:TerminateInstances",
        "iam:CreateRole",
        "s3:Getaway",
        "s3:Listen",
    ] {
        let source = catalog(&[("op", vec![action])]);
        let error = render(&source, UnmappedPolicy::Refuse)
            .err()
            .ok_or_else(|| format!("{action} must be refused"))?;
        assert!(
            matches!(error, PolicyError::NotReadOnly { .. }),
            "{action}: {error}"
        );
    }
    Ok(())
}

#[test]
fn read_verbs_are_allowed_including_batch_get_and_search() -> TestResult {
    let source = catalog(&[(
        "op",
        vec![
            "dynamodb:BatchGetItem",
            "resource-explorer-2:Search",
            "config:SelectAggregateResourceConfig",
            "s3:GetObjectTagging",
            "ec2:DescribeVpcs",
        ],
    )]);
    assert_eq!(
        policy_actions(&render(&source, UnmappedPolicy::Refuse)?.text)?.len(),
        5
    );
    Ok(())
}

#[test]
fn malformed_actions_are_refused() -> TestResult {
    for action in [
        "s3:*",
        "s3ListBuckets",
        "S3:ListBuckets",
        "s3:List:Buckets",
        "s3:",
        ":ListBuckets",
        "s3:List Buckets",
    ] {
        let source = catalog(&[("op", vec![action])]);
        let error = render(&source, UnmappedPolicy::Refuse)
            .err()
            .ok_or_else(|| format!("{action} must be refused"))?;
        assert!(
            matches!(error, PolicyError::Malformed { .. }),
            "{action}: {error}"
        );
    }
    Ok(())
}

#[test]
fn output_bytes_do_not_depend_on_catalog_order() -> TestResult {
    let forward = catalog(&[
        ("a", vec!["s3:ListAllMyBuckets"]),
        ("b", vec!["ec2:DescribeVpcs", "ec2:DescribeSubnets"]),
    ]);
    let reversed = catalog(&[
        ("b", vec!["ec2:DescribeSubnets", "ec2:DescribeVpcs"]),
        ("a", vec!["s3:ListAllMyBuckets"]),
    ]);
    let first = render(&forward, UnmappedPolicy::Refuse)?.text;
    assert_eq!(first, render(&reversed, UnmappedPolicy::Refuse)?.text);
    assert_eq!(first, render(&forward, UnmappedPolicy::Refuse)?.text);
    assert!(first.ends_with("}\n"));
    Ok(())
}

#[test]
fn oversized_policy_is_refused_instead_of_truncated() -> TestResult {
    let names: Vec<String> = (0..400)
        .map(|index| format!("ec2:DescribeVeryLongActionNameNumber{index:04}Padding"))
        .collect();
    let actions: Vec<&str> = names.iter().map(String::as_str).collect();
    let source = catalog(&[("op", actions)]);
    let error = render(&source, UnmappedPolicy::Refuse)
        .err()
        .ok_or("oversized policy must be refused")?;
    assert!(matches!(error, PolicyError::TooLarge { limit: 6144, .. }));
    Ok(())
}

#[test]
fn write_then_check_round_trips_and_detects_drift() -> TestResult {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("catalog"))?;
    fs::write(
        dir.path().join("catalog/aws-operations.toml"),
        catalog(&[("s3.list_buckets", vec!["s3:ListAllMyBuckets"])]),
    )?;
    let policy_path = dir.path().join("catalog/iam-read-only-policy.json");

    run(dir.path(), ReportMode::Write, UnmappedPolicy::Refuse)?;
    run(dir.path(), ReportMode::Check, UnmappedPolicy::Refuse)?;

    fs::write(&policy_path, "{}\n")?;
    assert!(run(dir.path(), ReportMode::Check, UnmappedPolicy::Refuse).is_err());
    Ok(())
}
