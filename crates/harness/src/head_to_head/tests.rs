use super::*;
use std::{cell::Cell, future, task::Poll};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn build(
    names: &[String],
    samples: &[Sample],
    versions: Value,
    dataset: Value,
    machine: Value,
) -> Value {
    let services = vec!["ec2".into(), "s3".into()];
    let scopes = names
        .iter()
        .map(|name| (name.clone(), services.clone()))
        .collect();
    let seeded_counts = dataset
        .get("seeded_counts")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(kind, count)| count.as_u64().map(|count| (kind.clone(), count)))
        .collect();
    report::build(
        names,
        samples,
        &report::Evidence {
            versions,
            dataset,
            seeded_counts,
            machine,
            services,
            scopes,
        },
    )
}

#[test]
fn given_three_tools_when_runs_rotate_then_each_starts_once() -> TestResult {
    let tools = ["ours", "cloudquery", "steampipe"];
    let orders: Vec<Vec<_>> = (0..3)
        .map(|run| report::rotation(&tools, run).copied().collect())
        .collect();
    assert_eq!(
        orders,
        vec![
            vec!["ours", "cloudquery", "steampipe"],
            vec!["cloudquery", "steampipe", "ours"],
            vec!["steampipe", "ours", "cloudquery"]
        ]
    );
    assert_eq!(report::rotation(&tools, usize::MAX).count(), 3);
    assert_eq!(report::rotation::<&str>(&[], 1).count(), 0);
    Ok(())
}

#[test]
fn given_wall_times_when_summarized_then_median_handles_even_odd_and_empty() -> TestResult {
    assert_eq!(report::median(&[9.0, 1.0, 3.0]), Some(3.0));
    assert_eq!(report::median(&[9.0, 1.0, 3.0, 5.0]), Some(4.0));
    assert_eq!(report::median(&[0.0]), Some(0.0));
    assert_eq!(report::median(&[]), None);
    assert_eq!(report::median(&[f64::NAN]), None);
    assert_eq!(report::median(&[f64::INFINITY]), None);
    assert_eq!(report::median(&[-1.0]), None);
    Ok(())
}

fn success(tool: &str, run: usize, seconds: f64, counts: &[(&str, u64)]) -> Sample {
    Sample {
        tool: tool.into(),
        run,
        outcome: Outcome::Success {
            seconds,
            counts: counts
                .iter()
                .map(|(kind, count)| ((*kind).into(), *count))
                .collect(),
            api_calls: Some(42),
            left_processes: false,
        },
    }
}

#[test]
fn given_different_kinds_when_comparing_then_only_the_intersection_counts() -> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("vpc", 6), ("subnet", 6)]),
        success("other", 0, 3.0, &[("vpc", 6), ("bucket", 2)]),
    ];
    let names = ["ours".into(), "other".into()];
    let result = build(&names, &samples, json!({}), json!({}), json!({}));
    assert_eq!(result.get("common_kinds"), Some(&json!(["vpc"])));
    assert_eq!(
        result.get("not_compared_kinds"),
        Some(&json!(["bucket", "subnet"]))
    );
    assert_eq!(result.get("equal_common_counts"), Some(&json!(true)));
    assert_eq!(
        result.pointer("/tools/0/runs/0/common_counts"),
        Some(&json!({ "vpc": 6 }))
    );
    assert_eq!(
        result.pointer("/tools/1/runs/0/common_counts"),
        Some(&json!({ "vpc": 6 }))
    );
    Ok(())
}

#[test]
fn given_one_sided_extra_kind_when_reporting_then_comparison_has_no_timing() -> TestResult {
    let samples = [
        success(
            "ours",
            0,
            1.0,
            &[("AWS::S3::Bucket", 2), ("AWS::S3::AccessPoint", 1)],
        ),
        success("other", 0, 2.0, &[("AWS::S3::Bucket", 2)]),
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("equal_common_counts"), Some(&json!(true)));
    assert_eq!(
        result.get("not_compared_kinds"),
        Some(&json!(["AWS::S3::AccessPoint"]))
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(
        result.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(
        result.pointer("/tools/0/runs/0/wall_seconds"),
        Some(&json!(1.0))
    );
    assert_eq!(
        result.pointer("/tools/1/runs/0/wall_seconds"),
        Some(&json!(2.0))
    );
    Ok(())
}

#[test]
fn given_successful_runs_when_reporting_then_median_and_spread_are_exact() -> TestResult {
    let samples = [
        success("ours", 0, 9.0, &[("vpc", 6)]),
        success("ours", 1, 1.0, &[("vpc", 6)]),
        success("ours", 2, 3.0, &[("vpc", 6)]),
    ];
    let result = build(&["ours".into()], &samples, json!({}), json!({}), json!({}));
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(
            &json!({ "median_seconds": 3.0, "min_seconds": 1.0, "max_seconds": 9.0, "spread_seconds": 8.0 })
        )
    );
    assert_eq!(result.get("equal_common_counts"), Some(&json!(true)));
    assert_eq!(result.get("comparison_ready"), Some(&json!(true)));
    Ok(())
}

#[test]
fn given_failed_run_when_reporting_then_failure_has_no_timing_and_comparison_waits() -> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("vpc", 6)]),
        Sample {
            tool: "other".into(),
            run: 0,
            outcome: Outcome::Failed {
                reason: "collect exited exit status: 7; see collect.stderr".into(),
            },
        },
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({}),
        json!({}),
    );
    assert_eq!(
        result.pointer("/tools/1/runs/0"),
        Some(&json!({ "run": 0, "status": "failed",
        "reason": "collect exited exit status: 7; see collect.stderr" }))
    );
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(
        result.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(result.pointer("/tools/1/runs/0/wall_seconds"), None);
    assert_eq!(result.get("common_kinds"), Some(&json!([])));
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    Ok(())
}

#[test]
fn given_disjoint_or_missing_tools_when_intersecting_then_comparison_is_empty() -> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("vpc", 1)]),
        success("other", 0, 2.0, &[("bucket", 2)]),
    ];
    assert_eq!(
        report::common_kinds(&samples, &["ours".into(), "other".into()]),
        BTreeSet::new()
    );
    assert_eq!(
        report::common_kinds(&samples, &["absent".into()]),
        BTreeSet::new()
    );
    assert_eq!(report::common_kinds(&[], &[]), BTreeSet::new());
    Ok(())
}

#[test]
fn given_kind_missing_from_one_run_when_intersecting_then_it_is_not_comparable() -> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("vpc", 1), ("bucket", 2)]),
        success("ours", 1, 2.0, &[("vpc", 1)]),
    ];
    assert_eq!(
        report::common_kinds(&samples, &["ours".into()]),
        BTreeSet::from(["vpc".into()])
    );
    Ok(())
}

fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

#[test]
fn given_repeated_tool_flags_when_parsing_then_all_definitions_are_retained() -> TestResult {
    let arguments = arguments(&[
        "--work",
        "/tmp/work",
        "--ours",
        "/bin/ours",
        "--tool",
        "a=/tmp/a.toml",
        "--tool",
        "b=/tmp/b.toml",
        "--service",
        "ec2",
        "--service",
        "s3",
    ]);
    let args = Args::parse(&arguments)?;
    assert_eq!(
        args.tools,
        vec![
            ("a".into(), PathBuf::from("/tmp/a.toml")),
            ("b".into(), PathBuf::from("/tmp/b.toml"))
        ]
    );
    assert_eq!(args.runs, 3);
    assert_eq!(args.services, vec!["ec2", "s3"]);
    Ok(())
}

#[test]
fn given_zero_runs_when_parsing_then_exact_error_is_returned() -> TestResult {
    let arguments = arguments(&["--runs", "0"]);
    let error = Args::parse(&arguments).err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "--runs must be 1..100");
    Ok(())
}

#[test]
fn given_duplicate_names_when_parsing_then_exact_error_is_returned() -> TestResult {
    let arguments = arguments(&["--tool", "same=a.toml", "--tool", "same=b.toml"]);
    let error = Args::parse(&arguments).err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "duplicate tool name");
    Ok(())
}

#[test]
fn given_seed_oracle_when_counting_then_only_resources_contribute() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("oracle.tsv");
    std::fs::write(
        &path,
        "resource\tvpc\ta\tr\tv1\nresource\tvpc\ta\tr\tv2\nedge\tsubnet\ts1\tIN_VPC\tvpc\tv1\n",
    )?;
    assert_eq!(
        dataset_counts(&path)?,
        report::Counts::from([("vpc".into(), 2)])
    );
    Ok(())
}

#[test]
fn given_service_created_oracle_row_when_counting_then_its_kind_is_included() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("oracle.tsv");
    std::fs::write(
        &path,
        "resource\tAWS::Connect::ContactFlow\t123456789012\tus-east-1\tflow-9\tservice-created\n",
    )?;
    let counts = dataset_counts(&path)?;
    assert_eq!(
        counts,
        report::Counts::from([("AWS::Connect::ContactFlow".into(), 1)])
    );
    Ok(())
}

#[test]
fn given_unknown_resource_tag_when_counting_then_oracle_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("oracle.tsv");
    std::fs::write(
        &path,
        concat!(
            "resource\tAWS::Connect::ContactFlow\t123456789012\tus-east-1\tflow-1\n",
            "resource\tAWS::Connect::ContactFlow\t123456789012\tus-east-1\tflow-9\tunknown\n",
        ),
    )?;
    let result = dataset_counts(&path);
    assert!(matches!(result, Err(HarnessError::Invalid(_))));
    Ok(())
}

#[test]
fn given_edge_row_with_empty_field_when_counting_then_oracle_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("oracle.tsv");
    let rows = [
        "edge\t\ts1\tIN_VPC\tvpc\tv1",
        "edge\tsubnet\t\tIN_VPC\tvpc\tv1",
        "edge\tsubnet\ts1\t\tvpc\tv1",
        "edge\tsubnet\ts1\tIN_VPC\t\tv1",
        "edge\tsubnet\ts1\tIN_VPC\tvpc\t",
    ];
    rows.iter().try_for_each(|row| -> TestResult {
        std::fs::write(&path, format!("resource\tvpc\ta\tr\tv1\n{row}\n"))?;
        let error = dataset_counts(&path)
            .err()
            .ok_or("empty edge field passed")?;
        assert_eq!(error.to_string(), "malformed seed oracle row");
        Ok(())
    })
}

#[test]
fn given_unequal_counts_within_or_between_tools_then_comparison_is_not_ready() -> TestResult {
    let cases = [
        (
            vec!["ours".into(), "other".into()],
            vec![
                success("ours", 0, 1.0, &[("vpc", 6)]),
                success("other", 0, 2.0, &[("vpc", 5)]),
            ],
        ),
        (
            vec!["ours".into()],
            vec![
                success("ours", 0, 1.0, &[("vpc", 6)]),
                success("ours", 1, 2.0, &[("vpc", 5)]),
            ],
        ),
    ];
    cases
        .into_iter()
        .try_for_each(|(names, samples)| -> TestResult {
            let result = build(&names, &samples, json!({}), json!({}), json!({}));
            assert_eq!(result.get("equal_common_counts"), Some(&json!(false)));
            assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
            assert_eq!(
                result.pointer("/tools/0/comparison_timing"),
                Some(&Value::Null)
            );
            Ok(())
        })
}

#[test]
fn given_zero_row_tool_when_comparing_then_no_kinds_are_common() -> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[]),
        success("other", 0, 2.0, &[("vpc", 6)]),
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({}),
        json!({}),
    );
    assert_eq!(result.get("common_kinds"), Some(&json!([])));
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    Ok(())
}

#[test]
fn given_different_scope_when_reporting_then_equal_counts_do_not_enable_timing() -> TestResult {
    let names = ["ours".into(), "other".into()];
    let samples = [
        success("ours", 0, 1.0, &[("vpc", 6)]),
        success("other", 0, 2.0, &[("vpc", 6)]),
    ];
    let evidence = report::Evidence {
        versions: json!({}),
        dataset: json!({}),
        seeded_counts: report::Counts::new(),
        machine: json!({}),
        services: vec!["ec2".into(), "s3".into()],
        scopes: std::collections::BTreeMap::from([
            ("ours".into(), vec!["ec2".into(), "s3".into()]),
            ("other".into(), vec!["ec2".into()]),
        ]),
    };
    let result = report::build(&names, &samples, &evidence);
    assert_eq!(result.get("equal_common_counts"), Some(&json!(true)));
    assert_eq!(result.get("equal_services"), Some(&json!(false)));
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(result.get("services"), Some(&json!(["ec2", "s3"])));
    assert_eq!(result.pointer("/tools/1/services"), Some(&json!(["ec2"])));
    assert_eq!(
        result.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    Ok(())
}

fn complete_arguments() -> Vec<String> {
    arguments(&[
        "--work",
        "/tmp/work",
        "--ours",
        "/bin/ours",
        "--tool",
        "a=/tmp/a.toml",
    ])
}

#[test]
fn given_run_bounds_when_parsing_then_only_one_through_one_hundred_are_accepted() -> TestResult {
    ["0", "101", "not-a-number"]
        .into_iter()
        .try_for_each(|runs| -> TestResult {
            let input = complete_arguments()
                .into_iter()
                .chain(arguments(&["--runs", runs]))
                .collect::<Vec<_>>();
            assert_eq!(
                Args::parse(&input)
                    .err()
                    .ok_or("expected an error")?
                    .to_string(),
                "--runs must be 1..100"
            );
            Ok(())
        })?;
    [("1", 1), ("100", 100)]
        .into_iter()
        .try_for_each(|(runs, expected)| -> TestResult {
            let input = complete_arguments()
                .into_iter()
                .chain(arguments(&["--runs", runs]))
                .collect::<Vec<_>>();
            assert_eq!(Args::parse(&input)?.runs, expected);
            Ok(())
        })
}

#[test]
fn given_excess_or_missing_cli_values_when_parsing_then_exact_errors_are_returned() -> TestResult {
    let excessive: Vec<_> = arguments(&["--work", "/tmp/work", "--ours", "/bin/ours"])
        .into_iter()
        .chain((0..32).flat_map(|index| ["--tool".into(), format!("tool{index}=/tmp/tool.toml")]))
        .collect();
    assert_eq!(
        Args::parse(&excessive)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "supply 1..31 --tool definitions"
    );
    [
        arguments(&["--work"]),
        arguments(&["--work", "/tmp/work", "--tool", "a=/tmp/a.toml"]),
    ]
    .into_iter()
    .try_for_each(|input| -> TestResult {
        assert_eq!(
            Args::parse(&input)
                .err()
                .ok_or("expected an error")?
                .to_string(),
            USAGE
        );
        Ok(())
    })
}

#[test]
fn given_two_runs_and_three_tools_when_scheduling_then_run_indices_follow_rotation() -> TestResult {
    let tools = ["ours", "cloudquery", "steampipe"];
    let actual: Vec<_> = schedule(&tools, 2)
        .map(|(run, tool)| (run, *tool))
        .collect();
    assert_eq!(
        actual,
        vec![
            (0, "ours"),
            (0, "cloudquery"),
            (0, "steampipe"),
            (1, "cloudquery"),
            (1, "steampipe"),
            (1, "ours"),
        ]
    );
    Ok(())
}

#[test]
fn given_main_and_cleanup_failures_when_finishing_then_both_messages_are_preserved() -> TestResult {
    let error = finish_result::<()>(
        Err(HarnessError::invalid("collection failed")),
        Err(HarnessError::invalid("database drop failed")),
    )
    .err()
    .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "collection failed; cleanup failed: database drop failed"
    );
    Ok(())
}

#[tokio::test]
async fn given_failed_version_probe_when_reporting_then_run_is_never_polled_or_timed() -> TestResult
{
    let versions = json!({"other": {"error": "version exited exit status: 7; see version.stderr"}});
    let polled = std::sync::atomic::AtomicBool::new(false);
    let run = async {
        polled.store(true, std::sync::atomic::Ordering::SeqCst);
        Outcome::Failed {
            reason: "run was polled".into(),
        }
    };
    let outcome = gated_sample(&versions, "other", run).await;
    assert!(!polled.load(std::sync::atomic::Ordering::SeqCst));
    let samples = [Sample {
        tool: "other".into(),
        run: 0,
        outcome,
    }];
    let result = build(&["other".into()], &samples, versions, json!({}), json!({}));
    assert_eq!(
        result.pointer("/tools/0/runs/0"),
        Some(&json!({
            "run": 0, "status": "failed",
            "reason": "version command failed: version exited exit status: 7; see version.stderr; comparison waits for J3",
        }))
    );
    assert_eq!(result.pointer("/tools/0/runs/0/wall_seconds"), None);
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(
        version_failure(&json!({"other": {"output": "v1"}}), "other"),
        None
    );
    Ok(())
}

#[test]
fn given_thirty_one_tools_when_parsing_then_the_boundary_is_accepted() -> TestResult {
    let input: Vec<_> = arguments(&["--work", "/tmp/work", "--ours", "/bin/ours"])
        .into_iter()
        .chain((0..31).flat_map(|index| ["--tool".into(), format!("tool{index}=/tmp/tool.toml")]))
        .collect();
    let expected: Vec<_> = (0..31)
        .map(|index| (format!("tool{index}"), PathBuf::from("/tmp/tool.toml")))
        .collect();
    assert_eq!(Args::parse(&input)?.tools, expected);
    Ok(())
}

#[test]
fn given_unknown_flag_or_empty_work_when_parsing_then_usage_is_returned() -> TestResult {
    for input in [
        arguments(&["--unknown", "value"]),
        arguments(&["--work", ""]),
    ] {
        assert_eq!(
            Pairs::parse(&input)
                .err()
                .ok_or("expected an error")?
                .to_string(),
            USAGE
        );
    }
    Ok(())
}

#[test]
fn given_duplicate_work_flags_when_parsing_then_arguments_are_rejected() -> TestResult {
    let mut input = complete_arguments();
    input.extend(arguments(&["--work", "/tmp/other-work"]));
    assert!(matches!(Args::parse(&input), Err(HarnessError::Invalid(_))));
    Ok(())
}

#[tokio::test]
async fn given_stop_request_when_comparison_is_pending_then_shutdown_finishes_before_error()
-> TestResult {
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let waiting = future::pending::<Result<(), HarnessError>>();
    let result = until_stop(waiting, future::ready(Ok(()))).await;
    let error = finish_stop(
        result,
        async {
            stopped.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        },
        future::pending(),
    )
    .await
    .err()
    .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "head-to-head run interrupted");
    assert!(stopped.load(std::sync::atomic::Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn given_comparison_and_signal_ready_when_selecting_then_interruption_wins() -> TestResult {
    let comparison = future::ready(Ok(()));
    let signal = future::ready(Ok(()));
    let outcome = until_stop(comparison, signal).await;
    let result = finish_stop(outcome, future::ready(Ok(())), future::pending()).await;
    assert!(matches!(result, Err(HarnessError::Invalid(_))));
    Ok(())
}

#[tokio::test]
async fn given_signal_during_shutdown_when_finishing_then_drain_precedes_error() -> TestResult {
    let draining = Cell::new(false);
    let drained = Cell::new(false);
    let signal = future::poll_fn(|_| {
        if draining.get() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    });
    let cleanup = future::poll_fn(|cx| {
        if draining.replace(true) {
            drained.set(true);
            return Poll::Ready(Ok(()));
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    });
    let result = finish_stop(ControlFlow::Continue(Ok(())), cleanup, signal).await;
    assert!(matches!(result, Err(HarnessError::Invalid(_))));
    assert!(drained.get());
    Ok(())
}

#[tokio::test]
async fn given_signal_ready_on_final_cleanup_poll_then_finishing_returns_error() {
    let finished = Cell::new(false);
    let signal = future::poll_fn(|_| {
        if finished.get() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    });
    let cleanup = async {
        finished.set(true);
        Ok(())
    };
    let result = finish_stop(ControlFlow::Continue(Ok(())), cleanup, signal).await;
    assert!(matches!(result, Err(HarnessError::Invalid(_))));
}

#[test]
fn given_seeded_kind_below_oracle_when_both_tools_agree_then_samples_fail_without_timing()
-> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("AWS::S3::Bucket", 0)]),
        success("other", 0, 2.0, &[("AWS::S3::Bucket", 0)]),
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    for (tool, seconds) in [(0, 1.0), (1, 2.0)] {
        assert_eq!(
            result.pointer(&format!("/tools/{tool}/runs/0")),
            Some(&json!({
                "run": 0, "status": "failed",
                "reason": "seeded kind AWS::S3::Bucket: observed 0, requires exactly 2",
                "wall_seconds": seconds, "api_calls": 42, "common_counts": {},
                "observed_counts": {"AWS::S3::Bucket": 0},
                "left_processes_running": false, "process_cleanup": null,
            }))
        );
        assert_eq!(
            result.pointer(&format!("/tools/{tool}/comparison_timing")),
            Some(&Value::Null)
        );
    }
    Ok(())
}

#[test]
fn given_seeded_kind_absent_when_both_tools_agree_on_another_kind_then_samples_fail() -> TestResult
{
    let samples = [
        success("ours", 0, 1.0, &[("AWS::EC2::VPC", 1)]),
        success("other", 0, 2.0, &[("AWS::EC2::VPC", 1)]),
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    for tool in 0..2 {
        let run = format!("/tools/{tool}/runs/0");
        assert_eq!(
            result.pointer(&format!("{run}/status")),
            Some(&json!("failed"))
        );
        assert_eq!(
            result.pointer(&format!("{run}/reason")),
            Some(&json!(
                "seeded kind AWS::S3::Bucket: missing, requires exactly 2"
            ))
        );
        assert_eq!(
            result.pointer(&format!("{run}/observed_counts/AWS::EC2::VPC")),
            Some(&json!(1))
        );
        assert_eq!(
            result.pointer(&format!("/tools/{tool}/comparison_timing")),
            Some(&Value::Null)
        );
    }
    Ok(())
}

#[test]
fn given_seeded_counts_met_when_both_tools_agree_then_timings_remain_comparable() -> TestResult {
    for (kind, observed) in [
        ("AWS::S3::Bucket", 2),
        ("AWS::EC2::VPC", 2),
        ("AWS::EC2::VPC", 3),
        ("AWS::EC2::Subnet", 2),
        ("AWS::EC2::Subnet", 3),
    ] {
        let samples = [
            success("ours", 0, 1.0, &[(kind, observed)]),
            success("other", 0, 2.0, &[(kind, observed)]),
        ];
        let result = build(
            &["ours".into(), "other".into()],
            &samples,
            json!({}),
            json!({"seeded_counts": {kind: 2}}),
            json!({}),
        );
        assert_eq!(result.get("comparison_ready"), Some(&json!(true)));
        assert_eq!(
            result.pointer("/tools/0/runs/0/status"),
            Some(&json!("success"))
        );
        assert_eq!(
            result.pointer("/tools/1/comparison_timing/median_seconds"),
            Some(&json!(2.0))
        );
    }
    Ok(())
}

#[test]
fn given_vpc_or_subnet_count_below_oracle_when_tools_agree_then_comparison_is_rejected()
-> TestResult {
    for kind in ["AWS::EC2::VPC", "AWS::EC2::Subnet"] {
        let samples = [
            success("ours", 0, 1.0, &[(kind, 1)]),
            success("other", 0, 2.0, &[(kind, 1)]),
        ];
        let result = build(
            &["ours".into(), "other".into()],
            &samples,
            json!({}),
            json!({"seeded_counts": {kind: 2}}),
            json!({}),
        );
        assert_eq!(
            result.get("comparison_ready"),
            Some(&json!(false)),
            "{kind}"
        );
        for tool in 0..2 {
            let run = format!("/tools/{tool}/runs/0");
            assert_eq!(
                result.pointer(&format!("{run}/status")),
                Some(&json!("failed"))
            );
            assert_eq!(
                result.pointer(&format!("{run}/reason")),
                Some(&json!(format!(
                    "seeded kind {kind}: observed 1, requires at least 2"
                )))
            );
            assert_eq!(
                result.pointer(&format!("{run}/observed_counts/{kind}")),
                Some(&json!(1))
            );
            assert_eq!(
                result.pointer(&format!("/tools/{tool}/comparison_timing")),
                Some(&Value::Null)
            );
        }
    }
    Ok(())
}

#[test]
fn given_later_run_below_seeded_count_when_other_runs_pass_then_comparison_is_deferred()
-> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("AWS::S3::Bucket", 2)]),
        success("other", 0, 2.0, &[("AWS::S3::Bucket", 2)]),
        success("other", 1, 3.0, &[("AWS::S3::Bucket", 1)]),
        success("ours", 1, 4.0, &[("AWS::S3::Bucket", 2)]),
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(
        result.pointer("/tools/0/runs/1/status"),
        Some(&json!("success"))
    );
    assert_eq!(
        result.pointer("/tools/1/runs/1/status"),
        Some(&json!("failed"))
    );
    assert_eq!(
        result.pointer("/tools/1/runs/1/reason"),
        Some(&json!(
            "seeded kind AWS::S3::Bucket: observed 1, requires exactly 2"
        ))
    );
    assert_eq!(
        result.pointer("/tools/1/runs/1/wall_seconds"),
        Some(&json!(3.0))
    );
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(&Value::Null)
    );
    Ok(())
}

#[test]
fn given_extra_seeded_s3_bucket_when_tools_agree_then_comparison_is_rejected() -> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("AWS::S3::Bucket", 3)]),
        success("other", 0, 2.0, &[("AWS::S3::Bucket", 3)]),
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(
        result.pointer("/tools/0/runs/0"),
        Some(&json!({
            "run": 0, "status": "failed",
            "reason": "seeded kind AWS::S3::Bucket: observed 3, requires exactly 2",
            "wall_seconds": 1.0, "api_calls": 42, "common_counts": {},
            "observed_counts": {"AWS::S3::Bucket": 3},
            "left_processes_running": false, "process_cleanup": null,
        }))
    );
    assert_eq!(
        result.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    Ok(())
}

#[test]
fn given_zero_samples_when_building_report_then_comparison_has_no_timing() -> TestResult {
    let result = build(
        &["ours".into(), "other".into()],
        &[],
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(result.get("common_kinds"), Some(&json!([])));
    assert_eq!(result.pointer("/tools/0/runs"), Some(&json!([])));
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(&Value::Null)
    );
    Ok(())
}

#[test]
fn given_failed_sample_when_other_tool_succeeds_then_report_preserves_failure_without_timing()
-> TestResult {
    let samples = [
        success("ours", 0, 1.0, &[("AWS::S3::Bucket", 2)]),
        Sample {
            tool: "other".into(),
            run: 0,
            outcome: Outcome::Failed {
                reason: "collector refused".into(),
            },
        },
    ];
    let result = build(
        &["ours".into(), "other".into()],
        &samples,
        json!({}),
        json!({"seeded_counts": {"AWS::S3::Bucket": 2}}),
        json!({}),
    );
    assert_eq!(result.get("comparison_ready"), Some(&json!(false)));
    assert_eq!(
        result.pointer("/tools/1/runs/0"),
        Some(&json!({
            "run": 0, "status": "failed", "reason": "collector refused",
        }))
    );
    assert_eq!(
        result.pointer("/tools/0/comparison_timing"),
        Some(&Value::Null)
    );
    assert_eq!(
        result.pointer("/tools/1/comparison_timing"),
        Some(&Value::Null)
    );
    Ok(())
}

#[test]
fn given_empty_seed_oracle_when_comparing_then_run_is_refused() -> TestResult {
    let root = tempfile::tempdir()?;
    let oracle = root.path().join("oracle.tsv");
    std::fs::write(&oracle, "")?;
    let error = dataset_counts(&oracle).err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "head-to-head seed produced no resources");
    Ok(())
}
