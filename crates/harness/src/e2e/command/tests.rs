use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn parse(arguments: &[&str]) -> Result<RunArgs, HarnessError> {
    let arguments = arguments
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    RunArgs::parse(&arguments)
}

fn assert_invalid(arguments: &[&str]) {
    assert!(matches!(
        parse(arguments),
        Err(HarnessError::Invalid(reason)) if reason == USAGE
    ));
}

#[test]
fn given_repeated_service_options_when_parsed_then_every_selector_accumulates_in_order()
-> TestResult {
    let args = parse(&[
        "run",
        "--work",
        "work",
        "--service",
        "ec2:DescribeVpcs",
        "--service",
        "lambda",
        "--service",
        "ec2:DescribeSubnets",
        "--service",
        "lambda",
    ])?;
    assert_eq!(
        args.services,
        [
            "ec2:DescribeVpcs",
            "lambda",
            "ec2:DescribeSubnets",
            "lambda"
        ]
    );
    assert_eq!(args.work, PathBuf::from("work"));
    Ok(())
}

#[test]
fn given_unknown_option_when_parsed_then_harness_returns_invalid_usage() {
    assert_invalid(&["run", "--work", "work", "--unknown", "value"]);
    assert_invalid(&["run", "--work", "work", "--unknown"]);
}

#[test]
fn given_empty_option_value_when_parsed_then_harness_rejects_it() {
    [
        "--work",
        "--seeds",
        "--collector",
        "--service",
        "--postgres-url",
    ]
    .iter()
    .for_each(|option| assert_invalid(&["run", "--work", "work", option, ""]));
}

#[test]
fn given_missing_work_option_when_parsed_then_harness_returns_invalid_usage() {
    assert_invalid(&["run"]);
    assert_invalid(&["run", "--keep"]);
    assert_invalid(&["run", "--service", "ec2"]);
}
