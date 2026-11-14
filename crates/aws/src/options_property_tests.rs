use std::collections::BTreeSet;

use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError};

use super::{AwsScanError, AwsScanOptions, validate_region};

const SELECTORS: &[(&str, &[&str])] = &[
    ("ec2", &["ec2", "network"]),
    ("vpc", &["ec2", "network"]),
    ("ebs", &["ec2", "network"]),
    ("network", &["network"]),
    ("dynamodb", &["dynamodb"]),
    ("dynamo", &["dynamodb"]),
    ("lambda", &["lambda"]),
    ("ecs", &["ecs", "ecs-task-definitions"]),
    ("ecs-task-definitions", &["ecs-task-definitions"]),
    ("eks", &["eks"]),
    ("elbv2", &["elbv2"]),
    ("elb", &["elbv2"]),
    ("load-balancing", &["elbv2"]),
    ("s3", &["s3"]),
    ("iam", &["iam"]),
    ("route53", &["route53"]),
    ("cloudfront", &["cloudfront"]),
    ("sqs", &["sqs"]),
    ("sns", &["sns"]),
    ("kms", &["kms"]),
    ("step-functions", &["step-functions"]),
    ("stepfunctions", &["step-functions"]),
    ("sfn", &["step-functions"]),
];

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x4157_535f_4f50_5453),
        ..Config::default()
    }
}

fn options() -> AwsScanOptions {
    AwsScanOptions {
        profile: None,
        regions: vec!["us-east-1".into()],
        services: vec!["iam".into()],
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: false,
        assume_role: None,
        cloud_control: false,
        selection: inventory_core::Selection::Types(Vec::new()),
        max_pages: 1,
        page_size: 100,
    }
}

fn expected(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn all_native() -> BTreeSet<String> {
    expected(&[
        "ec2",
        "network",
        "dynamodb",
        "lambda",
        "ecs",
        "ecs-task-definitions",
        "eks",
        "elbv2",
        "s3",
        "iam",
        "route53",
        "cloudfront",
        "sqs",
        "sns",
        "kms",
        "step-functions",
    ])
}

fn admitted(options: &AwsScanOptions) -> Result<BTreeSet<String>, TestCaseError> {
    options
        .validate()
        .map_err(|error| TestCaseError::fail(format!("valid input was rejected: {error}")))
}

fn rejected(options: &AwsScanOptions, message: &str) -> Result<(), TestCaseError> {
    match options.validate() {
        Err(AwsScanError::InvalidOptions(actual)) if actual == message => Ok(()),
        actual => Err(TestCaseError::fail(format!(
            "expected InvalidOptions({message:?}), received {actual:?}"
        ))),
    }
}

#[derive(Clone, Copy, Debug)]
enum OptionalText {
    Profile,
    ConfigAggregator,
    ResourceExplorerView,
}

impl OptionalText {
    fn assign(self, options: &mut AwsScanOptions, value: Option<String>) {
        match self {
            Self::Profile => options.profile = value,
            Self::ConfigAggregator => options.config_aggregator = value,
            Self::ResourceExplorerView => options.resource_explorer_view = value,
        }
    }

    fn error(self) -> &'static str {
        match self {
            Self::Profile => "invalid profile",
            Self::ConfigAggregator => "invalid Config aggregator",
            Self::ResourceExplorerView => "invalid Resource Explorer view",
        }
    }
}

const TEXT_FIELDS: [OptionalText; 3] = [
    OptionalText::Profile,
    OptionalText::ConfigAggregator,
    OptionalText::ResourceExplorerView,
];

fn text_with_bytes(bytes: usize, unit: &str) -> String {
    format!(
        "{}{}",
        unit.repeat(bytes.div_euclid(unit.len())),
        "x".repeat(bytes.rem_euclid(unit.len()))
    )
}

#[test]
fn each_named_selector_and_alias_has_its_documented_collector_set() -> Result<(), AwsScanError> {
    for (selector, collectors) in SELECTORS {
        let mut options = options();
        options.services = vec![(*selector).to_owned()];
        assert_eq!(options.validate()?, expected(collectors), "{selector}");
    }
    let mut options = options();
    options.services.clear();
    assert_eq!(options.validate()?, all_native());
    options.services = vec!["all".into()];
    assert_eq!(options.validate()?, all_native());
    Ok(())
}

#[test]
fn supplemental_selectors_require_their_own_explicit_or_discovered_source()
-> Result<(), AwsScanError> {
    for explicit in [false, true] {
        for discover in [false, true] {
            for selector in ["config", "resource-explorer"] {
                let mut options = options();
                options.services = vec![selector.into()];
                options.discover_sources = discover;
                match selector {
                    "config" => {
                        options.resource_explorer_view = Some("fixture-view".into());
                        options.config_aggregator = explicit.then(|| "fixture-aggregator".into());
                    }
                    _ => {
                        options.config_aggregator = Some("fixture-aggregator".into());
                        options.resource_explorer_view = explicit.then(|| "fixture-view".into());
                    }
                }
                if explicit || discover {
                    assert!(options.validate()?.is_empty(), "{selector}");
                } else {
                    assert!(matches!(
                        options.validate(),
                        Err(AwsScanError::InvalidOptions(_))
                    ));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn cloud_control_selector_requires_its_flag_even_when_other_sources_exist()
-> Result<(), AwsScanError> {
    let mut options = options();
    options.services = vec!["cloud-control".into()];
    options.discover_sources = true;
    options.config_aggregator = Some("fixture-aggregator".into());
    options.resource_explorer_view = Some("fixture-view".into());
    assert!(matches!(
        options.validate(),
        Err(AwsScanError::InvalidOptions(_))
    ));
    options.cloud_control = true;
    assert!(options.validate()?.is_empty());
    Ok(())
}

#[test]
fn page_bounds_are_inclusive_and_independent() -> Result<(), AwsScanError> {
    for max_pages in [1, 10_000] {
        for page_size in [1, 1_000] {
            let mut options = options();
            options.max_pages = max_pages;
            options.page_size = page_size;
            assert_eq!(options.validate()?, expected(&["iam"]));
        }
    }
    for max_pages in [0, 10_001, u32::MAX] {
        let mut options = options();
        options.max_pages = max_pages;
        assert!(matches!(
            options.validate(),
            Err(AwsScanError::InvalidOptions(_))
        ));
    }
    for page_size in [i32::MIN, -1, 0, 1_001, i32::MAX] {
        let mut options = options();
        options.page_size = page_size;
        assert!(matches!(
            options.validate(),
            Err(AwsScanError::InvalidOptions(_))
        ));
    }
    Ok(())
}

mod region_count_limit_is_inclusive_before_deduplication;
