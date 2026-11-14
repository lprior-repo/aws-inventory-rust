use std::collections::BTreeSet;

use super::{
    EXACT_CASES, Exceptions, MAX_SQL_NAME_BYTES, NameError, TestResult, render_names, sql_name,
};

const ORACLE: &str = include_str!("../../../tests/fixtures/names-plain.tsv");
const EXCEPTIONS: &str = include_str!("../../../../../catalog/name-exceptions.tsv");
const CATALOG: &str = include_str!("../../../../../catalog/cloudcontrol-universal.json");

#[derive(serde::Deserialize)]
struct Catalog {
    resource_types: Vec<ResourceType>,
}

#[derive(serde::Deserialize)]
struct ResourceType {
    type_name: String,
}

fn oracle_rows() -> Result<Vec<(&'static str, &'static str)>, Box<dyn std::error::Error>> {
    ORACLE
        .lines()
        .map(|line| {
            line.split_once('\t')
                .ok_or_else(|| format!("invalid oracle row: {line:?}").into())
        })
        .collect()
}

#[test]
fn plain_names_match_all_independent_oracle_rows_reporting_every_difference() -> TestResult {
    let rows = oracle_rows()?;
    assert_eq!(rows.len(), 1_881);
    let differences: Vec<_> = rows
        .iter()
        .filter_map(
            |(kind, expected)| match sql_name(kind, &Exceptions::none()) {
                Ok(name) if name.as_str() == *expected => None,
                actual => Some(format!("{kind}: oracle={expected}, contract={actual:?}")),
            },
        )
        .collect();
    assert_eq!(
        differences,
        Vec::<String>::new(),
        "{}",
        differences.join("\n")
    );
    Ok(())
}

#[test]
fn reviewed_exceptions_render_1881_unique_bounded_names_in_type_order() -> TestResult {
    let catalog: Catalog = serde_json::from_str(CATALOG)?;
    let types: Vec<_> = catalog
        .resource_types
        .iter()
        .map(|kind| kind.type_name.as_str())
        .collect();
    let mut expected: Vec<_> = oracle_rows()?.into_iter().map(|(kind, _)| kind).collect();
    expected.sort_unstable();
    let rows = render_names(&types, &Exceptions::parse(EXCEPTIONS)?)?;
    assert_eq!(
        rows.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(),
        expected
    );
    assert_catalog_names(&rows);
    Ok(())
}

fn assert_catalog_names(rows: &[(&str, super::super::SqlName)]) {
    assert_eq!(rows.len(), 1_881);
    let unique = rows.iter().map(|(_, name)| name).collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 1_881);
    let oversized = rows
        .iter()
        .filter(|(_, name)| name.as_str().len() > MAX_SQL_NAME_BYTES);
    assert_eq!(oversized.count(), 0);
    EXACT_CASES.iter().for_each(|(kind, expected)| {
        let actual = rows.iter().find(|(found, _)| found == kind);
        assert_eq!(
            actual.map(|(_, name)| name.as_str()),
            Some(*expected),
            "{kind}"
        );
    });
}

#[test]
fn rendering_sorts_types_regardless_of_input_order() -> TestResult {
    let rows = render_names(&["Z::Thing", "A::Thing"], &Exceptions::none())?;
    let actual: Vec<_> = rows
        .iter()
        .map(|(kind, name)| (*kind, name.as_str()))
        .collect();
    assert_eq!(actual, [("A::Thing", "a_thing"), ("Z::Thing", "z_thing")]);
    Ok(())
}

#[test]
fn rendering_reports_the_first_invalid_type_in_input_order() -> TestResult {
    assert_eq!(
        render_names(&["Z", "A"], &Exceptions::none()),
        Err(NameError::InvalidType {
            name: "Z".to_owned()
        })
    );
    Ok(())
}

#[test]
fn rendering_reports_colliding_plain_names_with_both_types() -> TestResult {
    assert_eq!(
        render_names(
            &["AWS::EC2::EC2Fleet", "AWS::EC2::Ec2Fleet"],
            &Exceptions::none()
        ),
        Err(NameError::DuplicateSqlName {
            name: sql_name("AWS::EC2::EC2Fleet", &Exceptions::none())?,
            first_type: "AWS::EC2::EC2Fleet".to_owned(),
            second_type: "AWS::EC2::Ec2Fleet".to_owned(),
        })
    );
    Ok(())
}

#[test]
fn rendering_reports_collisions_introduced_by_exceptions() -> TestResult {
    let exceptions = Exceptions::parse("X\tsame\nY\tsame\n")?;
    assert_eq!(
        render_names(&["AWS::X", "AWS::Y"], &exceptions),
        Err(NameError::DuplicateSqlName {
            name: sql_name("AWS::X", &exceptions)?,
            first_type: "AWS::X".to_owned(),
            second_type: "AWS::Y".to_owned(),
        })
    );
    Ok(())
}

#[test]
fn rendering_rejects_repeated_types_instead_of_dropping_rows() -> TestResult {
    assert_eq!(
        render_names(&["A::B", "A::B"], &Exceptions::none()),
        Err(NameError::DuplicateSqlName {
            name: sql_name("A::B", &Exceptions::none())?,
            first_type: "A::B".to_owned(),
            second_type: "A::B".to_owned(),
        })
    );
    Ok(())
}

#[test]
fn rendering_reports_first_unused_exception_in_original_line_order() -> TestResult {
    let exceptions = Exceptions::parse("IoT\tiot\nZ\tz\nA\ta\n")?;
    assert_eq!(
        render_names(&["AWS::IoT"], &exceptions),
        Err(NameError::UnusedException {
            line: 2,
            spelling: "Z".to_owned()
        })
    );
    Ok(())
}

#[test]
fn rendering_does_not_count_shadowed_shorter_exceptions_as_used() -> TestResult {
    let exceptions = Exceptions::parse("IP\tip\nIPv6\tipv6\n")?;
    assert_eq!(
        render_names(&["AWS::IPv6"], &exceptions),
        Err(NameError::UnusedException {
            line: 1,
            spelling: "IP".to_owned()
        })
    );
    Ok(())
}

#[test]
fn rendering_accumulates_exception_usage_across_types() -> TestResult {
    let exceptions = Exceptions::parse("IoT\tiot\nIP\tip\n")?;
    let rows = render_names(&["AWS::IoT", "AWS::IP"], &exceptions)?;
    assert_eq!(
        rows.iter()
            .map(|(_, name)| name.as_str())
            .collect::<Vec<_>>(),
        ["aws_ip", "aws_iot"]
    );
    Ok(())
}

#[test]
fn rendering_empty_input_requires_an_empty_exception_list() -> TestResult {
    let rows = render_names(&[], &Exceptions::none())?;
    assert_eq!(rows.len(), 0);
    let exceptions = Exceptions::parse("IoT\tiot\n")?;
    assert_eq!(
        render_names(&[], &exceptions),
        Err(NameError::UnusedException {
            line: 1,
            spelling: "IoT".to_owned()
        })
    );
    Ok(())
}

#[test]
fn rendering_propagates_overlong_names_before_unused_exception_errors() -> TestResult {
    let exceptions = Exceptions::parse("IoT\tiot\n")?;
    assert_eq!(
        render_names(&[&format!("A::{}", "B".repeat(127))], &exceptions),
        Err(NameError::TooLong {
            type_name: format!("A::{}", "B".repeat(127)),
            limit: MAX_SQL_NAME_BYTES
        })
    );
    Ok(())
}

const COMMITTED_EXCEPTION_CASES: [(&str, &str); 11] = [
    ("AWS::ACMPCA::Certificate", "aws_acm_pca_certificate"),
    ("AWS::B2BI::Capability", "aws_b2bi_capability"),
    ("AWS::FSx::FileSystem", "aws_fsx_file_system"),
    ("AWS::EC2::IPv4Pool", "aws_ec2_ipv4_pool"),
    ("AWS::IoT::Thing", "aws_iot_thing"),
    (
        "AWS::QuickSight::OAuthClientApplication",
        "aws_quick_sight_oauth_client_application",
    ),
    (
        "AWS::BedrockAgentCore::OAuth2CredentialProvider",
        "aws_bedrock_agent_core_oauth2_credential_provider",
    ),
    (
        "AWS::CloudWatch::OTelEnrichment",
        "aws_cloud_watch_otel_enrichment",
    ),
    (
        "AWS::SMSVOICE::ConfigurationSet",
        "aws_sms_voice_configuration_set",
    ),
    (
        "AWS::EC2::VPCDHCPOptionsAssociation",
        "aws_ec2_vpc_dhcp_options_association",
    ),
    ("AWS::WAFv2::WebACL", "aws_wafv2_web_acl"),
];

#[test]
fn committed_exceptions_preserve_each_reviewed_real_type_name() -> TestResult {
    let exceptions = Exceptions::parse(EXCEPTIONS)?;
    COMMITTED_EXCEPTION_CASES
        .iter()
        .try_for_each(|(kind, expected)| {
            let name = sql_name(kind, &exceptions)?;
            assert_eq!(name.as_str(), *expected, "{kind}");
            Ok(())
        })
}

#[test]
fn collision_payload_preserves_reversed_input_order() -> TestResult {
    assert_eq!(
        render_names(
            &["AWS::EC2::Ec2Fleet", "AWS::EC2::EC2Fleet"],
            &Exceptions::none()
        ),
        Err(NameError::DuplicateSqlName {
            name: sql_name("AWS::EC2::Ec2Fleet", &Exceptions::none())?,
            first_type: "AWS::EC2::Ec2Fleet".to_owned(),
            second_type: "AWS::EC2::EC2Fleet".to_owned(),
        })
    );
    Ok(())
}
