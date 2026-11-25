use super::{Args, endpoint, required};
use crate::probe::ProbeError;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn arguments() -> Vec<String> {
    [
        "--catalog",
        "probe.catalog",
        "--endpoint",
        "https://127.0.0.1:443",
        "--ca-bundle",
        "ca.pem",
        "--region",
        "us-east-1",
        "--report",
        "probe.report",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[test]
fn given_all_probe_options_when_parsed_then_paths_and_region_are_preserved() -> TestResult {
    let args = Args::parse(&arguments())?;
    assert_eq!(args.catalog, std::path::Path::new("probe.catalog"));
    assert_eq!(args.ca_bundle, std::path::Path::new("ca.pem"));
    assert_eq!(args.report, std::path::Path::new("probe.report"));
    assert_eq!(args.endpoint, "https://127.0.0.1:443");
    assert_eq!(args.region, "us-east-1");
    assert!(args.services.is_empty());
    Ok(())
}

#[test]
fn given_wrong_option_count_when_parsed_then_exact_arity_is_required() {
    for count in 0..10 {
        let args = arguments().into_iter().take(count).collect::<Vec<_>>();
        assert!(matches!(
            Args::parse(&args),
            Err(ProbeError::Invalid(
                "probe requires five option-value pairs and optional service pairs"
            ))
        ));
    }
    let mut args = arguments();
    args.push("extra".to_owned());
    assert!(matches!(
        Args::parse(&args),
        Err(ProbeError::Invalid(
            "probe requires five option-value pairs and optional service pairs"
        ))
    ));
}

#[test]
fn given_unknown_duplicate_or_empty_options_when_parsed_then_specific_errors_return() -> TestResult
{
    for (from, to, message) in [
        ("--report", "--unknown", "unknown probe option"),
        ("--report", "--catalog", "duplicate probe option"),
        (
            "probe.report",
            "",
            "probe option value is empty or too long",
        ),
    ] {
        let mut args = arguments();
        let value = args
            .iter_mut()
            .find(|value| value.as_str() == from)
            .ok_or("option missing")?;
        *value = to.to_owned();
        assert!(matches!(Args::parse(&args), Err(ProbeError::Invalid(found)) if found == message));
    }
    Ok(())
}

#[test]
fn given_oversized_option_when_parsed_then_value_bound_is_enforced() -> TestResult {
    let mut args = arguments();
    *args.last_mut().ok_or("report value missing")? = "x".repeat(4097);
    assert!(matches!(
        Args::parse(&args),
        Err(ProbeError::Invalid(
            "probe option value is empty or too long"
        ))
    ));
    Ok(())
}

#[test]
fn given_ipv4_or_ipv6_loopback_https_origin_when_validated_then_endpoint_is_accepted() {
    for value in [
        "https://127.0.0.1:1",
        "https://127.2.3.4:65535",
        "https://[::1]:443",
    ] {
        assert!(matches!(endpoint(value), Ok(())));
    }
}

#[test]
fn given_nonlocal_or_nonorigin_endpoint_when_validated_then_endpoint_is_rejected() {
    for value in [
        "http://127.0.0.1:443",
        "https://example.com:443",
        "https://localhost:443",
        "https://127.0.0.1",
        "https://127.0.0.1:0",
        "https://127.0.0.1:443/",
        "https://127.0.0.1:443/path",
        "https://127.0.0.1:443?query",
        "https://127.0.0.1:443#fragment",
        "https://user@127.0.0.1:443",
    ] {
        assert!(
            matches!(endpoint(value), Err(ProbeError::Invalid(_))),
            "{value}"
        );
    }
}

#[test]
fn given_missing_option_when_required_then_missing_option_error_returns() {
    assert!(matches!(
        required(None),
        Err(ProbeError::Invalid("missing required probe option"))
    ));
}

#[test]
fn given_repeated_services_when_parsed_then_each_selector_is_preserved() -> TestResult {
    let mut args = arguments();
    args.extend(["--service", "s3", "--service", "ec2"].map(str::to_owned));
    assert_eq!(Args::parse(&args)?.services, ["s3", "ec2"]);
    Ok(())
}

#[test]
fn given_too_many_services_when_parsed_then_argument_bound_is_enforced() {
    let mut args = arguments();
    args.extend(std::iter::repeat_n("--service".to_owned(), 514));
    assert!(Args::parse(&args).is_err());
}
