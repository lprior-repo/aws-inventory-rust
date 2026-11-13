//! Scan admission is checked without credentials or I/O. The expected selector
//! sets and limits below are public input contracts, independent of fixtures
//! that happen to exercise every collector through the `all` selector.

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
        resource_types: vec![],
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
        unit.repeat(bytes / unit.len()),
        "x".repeat(bytes % unit.len())
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
                        // An unrelated view cannot satisfy a Config source.
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

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn region_count_limit_is_inclusive_before_deduplication(region in "[a-z][a-z0-9-]{0,20}") {
        for count in [0, 1, 63, 64, 65, 66] {
            let mut options = options();
            options.regions = vec![region.clone(); count];
            if count <= 64 {
                prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
            } else {
                rejected(&options, "at most 64 regions and 32 service selectors are accepted")?;
            }
        }
    }

    #[test]
    fn selector_count_limit_is_inclusive_before_deduplication(index in 0_usize..SELECTORS.len()) {
        let (selector, collectors) = SELECTORS.get(index).ok_or_else(|| TestCaseError::fail("invalid fixture selector index"))?;
        for count in [1, 31, 32, 33, 34] {
            let mut options = options();
            options.services = vec![(*selector).to_owned(); count];
            if count <= 32 {
                prop_assert_eq!(admitted(&options)?, expected(collectors));
            } else {
                rejected(&options, "at most 64 regions and 32 service selectors are accepted")?;
            }
        }
    }

    #[test]
    fn valid_region_and_selector_caps_can_be_used_together(region in "[a-z][a-z0-9-]{0,20}") {
        let mut options = options();
        options.regions = vec![region; 64];
        options.services = vec!["iam".into(); 32];
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
    }

    #[test]
    fn selector_unions_are_order_and_duplicate_independent(indices in prop::collection::vec(0_usize..SELECTORS.len(), 1..=32)) {
        let mut options = options();
        options.services.clear();
        let mut selected = BTreeSet::new();
        for index in indices {
            let (selector, collectors) = SELECTORS.get(index).ok_or_else(|| TestCaseError::fail("invalid fixture selector index"))?;
            options.services.push((*selector).to_owned());
            selected.extend(expected(collectors));
        }
        prop_assert_eq!(admitted(&options)?, selected.clone());
        options.services.reverse();
        prop_assert_eq!(admitted(&options)?, selected);
        options.services = vec!["all".into(), "ecs".into(), "iam".into()];
        prop_assert_eq!(admitted(&options)?, all_native());
    }

    #[test]
    fn all_never_hides_an_unknown_selector(unknown in "unknown-[a-z0-9]{1,24}") {
        let mut options = options();
        options.services = vec!["all".into(), unknown.clone()];
        rejected(&options, &format!("unsupported service selector or missing supplemental source: {unknown}"))?;
    }

    #[test]
    fn resource_type_selection_requires_explicit_cloud_control(count in 1_usize..8) {
        let mut options = options();
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
        options.resource_types = vec!["AWS::EC2::VPC".into(); count];
        rejected(&options, "resource type selection requires Cloud Control")?;
        options.cloud_control = true;
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
        options.resource_types.clear();
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
    }

    #[test]
    fn optional_text_uses_exact_byte_limits_without_truncating_unicode(unit in prop::sample::select(vec!["a", "é", "中", "🦀"])) {
        for field in TEXT_FIELDS {
            let mut options = options();
            field.assign(&mut options, None);
            prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
            for bytes in [0, 1, 2_047, 2_048, 2_049, 2_050] {
                let value = text_with_bytes(bytes, unit);
                prop_assert_eq!(value.len(), bytes);
                field.assign(&mut options, Some(value));
                if (1..=2_048).contains(&bytes) {
                    prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
                } else {
                    rejected(&options, field.error())?;
                }
            }
        }
    }

    #[test]
    fn nul_in_any_optional_text_is_rejected_without_masking_by_other_limits(prefix in "[a-z0-9]{0,40}", suffix in "[a-z0-9]{0,40}") {
        for field in TEXT_FIELDS {
            let mut options = options();
            field.assign(&mut options, Some(format!("{prefix}x{suffix}")));
            prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
            field.assign(&mut options, Some(format!("{prefix}\0{suffix}")));
            rejected(&options, field.error())?;
        }
    }

    #[test]
    fn region_length_and_ascii_alphabet_are_independent_contracts(valid in "[a-z0-9-]{1,64}", bad in prop::sample::select(vec!["A", "_", " ", "\n", "\0", "é", "１"])) {
        prop_assert!(validate_region(&valid).is_ok());
        prop_assert!(validate_region("").is_err());
        prop_assert!(validate_region(&"a".repeat(64)).is_ok());
        prop_assert!(validate_region(&"a".repeat(65)).is_err());
        prop_assert!(validate_region(&format!("us{bad}east-1")).is_err(), "unsupported region character was admitted");
        let mut options = options();
        options.regions = vec![valid, format!("us{bad}east-1")];
        prop_assert!(matches!(options.validate(), Err(AwsScanError::InvalidOptions(_))), "invalid region was not rejected by scan admission");
    }
}
