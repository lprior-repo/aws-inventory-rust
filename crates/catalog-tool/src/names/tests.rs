use super::{Exceptions, MAX_SQL_NAME_BYTES, NameError, render_names, sql_name, sql_name_used};

mod bounds;
mod catalog;
mod parsing;
mod sql_safety;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const EXACT_CASES: [(&str, &str); 7] = [
    ("AWS::EC2::Instance", "aws_ec2_instance"),
    ("AWS::IAM::Role", "aws_iam_role"),
    ("AWS::KMS::Key", "aws_kms_key"),
    ("AWS::RDS::DBInstance", "aws_rds_db_instance"),
    ("AWS::EC2::VPCEndpoint", "aws_ec2_vpc_endpoint"),
    (
        "AWS::ElasticLoadBalancingV2::TargetGroup",
        "aws_elastic_load_balancing_v2_target_group",
    ),
    ("AWS::DynamoDB::Table", "aws_dynamo_db_table"),
];

const OTHER_PLAIN_CASES: [(&str, &str); 15] = [
    ("AWS::S3::Bucket", "aws_s3_bucket"),
    ("AWS::Route53::HostedZone", "aws_route53_hosted_zone"),
    (
        "AWS::EC2::VPCDHCPOptionsAssociation",
        "aws_ec2_vpcdhcp_options_association",
    ),
    ("AWS::IoT::Thing", "aws_io_t_thing"),
    ("AWS::WAFv2::WebACL", "aws_wa_fv2_web_acl"),
    ("AWS::EC2::IPAM", "aws_ec2_ipam"),
    ("AWS::SSM::Document", "aws_ssm_document"),
    ("Alexa::ASK::Skill", "alexa_ask_skill"),
    ("AWS::EC2::EC2Fleet", "aws_ec2_ec2_fleet"),
    (
        "AWS::Lambda::EventSourceMapping",
        "aws_lambda_event_source_mapping",
    ),
    ("AWS::EC2::Ec2Fleet", "aws_ec2_ec2_fleet"),
    ("A::B", "a_b"),
    ("AWS::X::ABCV12", "aws_x_abc_v12"),
    ("AWS::X::V2Thing", "aws_x_v2_thing"),
    ("AWS::X::ABV2Thing", "aws_x_abv2_thing"),
];

#[test]
fn plain_names_preserve_required_acronyms_and_boundaries() -> TestResult {
    // Given the seven V30 acceptance types, when naming without exceptions, then names are exact.
    EXACT_CASES.iter().try_for_each(|(kind, expected)| {
        let name = sql_name(kind, &Exceptions::none())?;
        assert_eq!(name.as_str(), *expected, "{kind}");
        Ok(())
    })
}

#[test]
fn plain_names_split_versions_and_keep_other_digits_attached() -> TestResult {
    // Given additional boundary cases, when naming them, then only V30 boundaries split.
    OTHER_PLAIN_CASES.iter().try_for_each(|(kind, expected)| {
        let name = sql_name(kind, &Exceptions::none())?;
        assert_eq!(name.as_str(), *expected, "{kind}");
        Ok(())
    })
}

const LONGEST_CASES: [(&str, &str); 5] = [
    ("AWS::EC2::IPv6Pool", "aws_ec2_ipv6_pool"),
    ("AWS::EC2::EC2IPv6Pool", "aws_ec2_ec2_ipv6_pool"),
    ("AWS::IoTWireless", "aws_iot_wireless"),
    ("AWS::XIoTY", "aws_x_iot_y"),
    ("AWS::IoTIPv6", "aws_iot_ipv6"),
];

const EXCEPTION_CASES: [(&str, &str); 4] = [
    (
        "AWS::EC2::VPCDHCPOptionsAssociation",
        "aws_ec2_vpc_dhcp_options_association",
    ),
    ("AWS::WAFv2::WebACL", "aws_wafv2_web_acl"),
    (
        "AWS::BedrockAgentCore::OAuth2CredentialProvider",
        "aws_bedrock_agent_core_oauth_2_credential_provider",
    ),
    ("AWS::IoT::Thing", "aws_iot_thing"),
];

const INVALID_TYPES: [&str; 13] = [
    "",
    "AWS",
    "AWS::",
    "::EC2",
    "AWS:::EC2",
    "AWS::EC2::in stance",
    "AWS::EC2::1Instance",
    "AWS::EC2::Înstance",
    "AWS::EC2::A_B",
    "1AWS::EC2",
    "AWS::EC2::A\n",
    "AWS::EC2::A\t",
    "AWS::EC2::A-B",
];

#[test]
fn exceptions_replace_longest_spelling_at_each_position() -> TestResult {
    // Given overlapping and interior spellings, when naming, then longest matches win.
    let exceptions = Exceptions::parse("IP\tip\nIPv6\tipv6\nIoT\tiot\n")?;
    LONGEST_CASES.iter().try_for_each(|(kind, expected)| {
        let name = sql_name(kind, &exceptions)?;
        assert_eq!(name.as_str(), *expected, "{kind}");
        Ok(())
    })
}

#[test]
fn exceptions_preserve_replacement_words_and_split_unmatched_digits() -> TestResult {
    // Given replacement pieces, when naming, then unmatched spans are independently split.
    let exceptions =
        Exceptions::parse("VPCDHCP\tvpc_dhcp\nWAFv2\twafv2\nOAuth\toauth\nIoT\tiot\n")?;
    EXCEPTION_CASES.iter().try_for_each(|(kind, expected)| {
        let name = sql_name(kind, &exceptions)?;
        assert_eq!(name.as_str(), *expected, "{kind}");
        Ok(())
    })
}

#[test]
fn invalid_types_are_rejected_before_exception_matching() -> TestResult {
    // Given invalid parts, when naming, then every invalid type is reported verbatim.
    INVALID_TYPES.iter().for_each(|kind| {
        assert_eq!(
            sql_name(kind, &Exceptions::none()),
            Err(NameError::InvalidType {
                name: (*kind).to_owned()
            })
        );
    });
    Ok(())
}

#[test]
fn names_accept_exactly_128_bytes_and_reject_129_bytes() -> TestResult {
    // Given the byte boundary, when naming, then the limit is inclusive.
    let accepted = sql_name(&format!("A::{}", "B".repeat(126)), &Exceptions::none())?;
    assert_eq!(accepted.as_str(), format!("a_{}", "b".repeat(126)));
    assert_eq!(accepted.as_str().len(), MAX_SQL_NAME_BYTES);
    assert_eq!(
        sql_name(&format!("A::{}", "B".repeat(127)), &Exceptions::none()),
        Err(NameError::TooLong {
            type_name: format!("A::{}", "B".repeat(127)),
            limit: MAX_SQL_NAME_BYTES
        }),
    );
    Ok(())
}

#[test]
fn exception_expansion_obeys_the_same_byte_limit() -> TestResult {
    // Given a length-expanding replacement, when naming, then the final output is bounded.
    let exceptions = Exceptions::parse(&format!("X\t{}\n", "a".repeat(127)))?;
    assert_eq!(
        sql_name("A::X", &exceptions),
        Err(NameError::TooLong {
            type_name: "A::X".to_owned(),
            limit: MAX_SQL_NAME_BYTES
        }),
    );
    Ok(())
}

#[test]
fn used_flags_follow_original_line_order_and_reset_each_call() -> TestResult {
    // Given overlaps and stale flags, when naming twice, then only each call's matches remain.
    let exceptions = Exceptions::parse("IP\tip\nIPv6\tipv6\nIoT\tiot\n")?;
    let mut used = vec![true; 9];
    let first = sql_name_used("AWS::IoTIPv6::IPv6Pool", &exceptions, &mut used)?;
    assert_eq!(first.as_str(), "aws_iot_ipv6_ipv6_pool");
    assert_eq!(used, [false, true, true]);
    let second = sql_name_used("AWS::EC2::Instance", &exceptions, &mut used)?;
    assert_eq!(second.as_str(), "aws_ec2_instance");
    assert_eq!(used, [false, false, false]);
    Ok(())
}

#[test]
fn used_flags_clear_even_when_type_validation_fails() -> TestResult {
    // Given stale usage, when the next type is invalid, then flags are reset, not inherited.
    let exceptions = Exceptions::parse("IoT\tiot\n")?;
    let mut used = vec![true];
    assert_eq!(
        sql_name_used("AWS", &exceptions, &mut used),
        Err(NameError::InvalidType {
            name: "AWS".to_owned()
        })
    );
    assert_eq!(used, [false]);
    Ok(())
}

#[test]
fn sql_name_display_and_value_traits_preserve_the_identifier() -> TestResult {
    // Given a derived name, when displaying and cloning, then the identifier is unchanged.
    let name = sql_name("AWS::EC2::Instance", &Exceptions::none())?;
    assert_eq!(name.to_string(), "aws_ec2_instance");
    assert_eq!(name.clone(), name);
    assert_eq!(
        name.cmp(&sql_name("AWS::IAM::Role", &Exceptions::none())?),
        std::cmp::Ordering::Less
    );
    Ok(())
}

#[test]
fn repeated_exception_spelling_is_replaced_every_time_in_one_part() -> TestResult {
    // Given a repeated spelling, when naming one part, then both matches use the exception.
    let exceptions = Exceptions::parse("IoT\tiot\n")?;
    let mut used = Vec::new();
    let name = sql_name_used("AWS::IoTIoT", &exceptions, &mut used)?;
    assert_eq!(name.as_str(), "aws_iot_iot");
    assert_eq!(used, [true]);
    Ok(())
}

#[test]
fn exception_expansion_accepts_exactly_128_output_bytes() -> TestResult {
    // Given an expanding replacement, when the output is exactly 128 bytes, then it succeeds.
    let replacement = "a".repeat(126);
    let exceptions = Exceptions::parse(&format!("X\t{replacement}\n"))?;
    let name = sql_name("A::X", &exceptions)?;
    assert_eq!(name.as_str(), format!("a_{replacement}"));
    assert_eq!(name.as_str().len(), 128);
    Ok(())
}

#[test]
fn lowercase_initial_letters_are_valid_in_every_type_part() -> TestResult {
    // Given lower-case initial letters, when naming, then validation accepts them unchanged.
    let name = sql_name("aws::ec2::instance", &Exceptions::none())?;
    assert_eq!(name.as_str(), "aws_ec2_instance");
    Ok(())
}

#[test]
fn exceptions_cannot_make_a_digit_initial_type_part_valid() -> TestResult {
    // Given a matching exception, when the original part starts with a digit, then it is invalid.
    let exceptions = Exceptions::parse("1X\tone_x\n")?;
    assert_eq!(
        sql_name("AWS::1X", &exceptions),
        Err(NameError::InvalidType {
            name: "AWS::1X".to_owned()
        })
    );
    Ok(())
}
