use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn ids(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn given_equal_identities_when_compared_then_all_match() -> TestResult {
    let listed = ids(&["a", "b"]);
    let stored = ids(&["a", "b"]);
    assert_eq!(
        compare(&listed, &stored)?,
        Comparison {
            matched: 2,
            differences: vec![]
        }
    );
    Ok(())
}

#[test]
fn given_missing_and_extra_when_compared_then_direction_and_identity_are_exact() -> TestResult {
    let listed = ids(&["common", "missing"]);
    let stored = ids(&["common", "extra"]);
    assert_eq!(
        compare(&listed, &stored)?,
        Comparison {
            matched: 1,
            differences: vec![
                Difference {
                    direction: Direction::Missing,
                    identity: "missing"
                },
                Difference {
                    direction: Direction::Extra,
                    identity: "extra"
                },
            ]
        }
    );
    Ok(())
}

#[test]
fn given_arn_in_sweep_when_identifier_is_resource_part_then_it_matches() -> TestResult {
    let listed = ids(&["bucket", "topic:child", "volume/vol-1"]);
    let stored = ids(&[
        "arn:aws:s3:::bucket",
        "arn:aws:sns:us-east-1:123456789012:topic:child",
        "arn:aws:ec2:us-east-1:123456789012:volume/vol-1",
    ]);
    assert_eq!(
        compare(&listed, &stored)?,
        Comparison {
            matched: 3,
            differences: vec![]
        }
    );
    Ok(())
}

#[test]
fn given_two_arns_when_only_one_identifier_exists_then_both_candidates_are_ambiguous() -> TestResult
{
    let listed = ids(&["bucket"]);
    let stored = ids(&["arn:aws:s3:::bucket", "arn:aws:s3:us-west-2::bucket"]);
    let error = compare(&listed, &stored)
        .err()
        .ok_or("expected an ambiguous identifier")?;
    assert_eq!(
        error,
        AmbiguousIdentity {
            identifier: "bucket".to_owned(),
            candidates: vec![
                "arn:aws:s3:::bucket".to_owned(),
                "arn:aws:s3:us-west-2::bucket".to_owned(),
            ],
        }
    );
    Ok(())
}

#[test]
fn given_arn_identifiers_when_accounts_differ_then_resource_parts_do_not_match() -> TestResult {
    let listed = ids(&["arn:aws:sns:us-east-1:111111111111:topic"]);
    let stored = ids(&["arn:aws:sns:us-east-1:222222222222:topic"]);
    let comparison = compare(&listed, &stored)?;
    assert_eq!(comparison.matched, 0);
    assert_eq!(comparison.differences.len(), 2);
    Ok(())
}

#[test]
fn given_prefix_rule_when_type_direction_and_prefix_match_then_explained() -> TestResult {
    let parsed =
        rules("AWS::SNS::Topic\tmissing_from_sweep:system-*\temulator owns system topics\n")?;
    let rule = parsed.first().ok_or("missing rule")?;
    assert!(rule.covers(
        "AWS::SNS::Topic",
        &Difference {
            direction: Direction::Missing,
            identity: "system-42"
        }
    ));
    assert!(!rule.covers(
        "AWS::SNS::Topic",
        &Difference {
            direction: Direction::Extra,
            identity: "system-42"
        }
    ));
    assert!(!rule.covers(
        "AWS::SNS::Topic",
        &Difference {
            direction: Direction::Missing,
            identity: "customer-42"
        }
    ));
    assert!(!rule.covers(
        "AWS::S3::Bucket",
        &Difference {
            direction: Direction::Missing,
            identity: "system-42"
        }
    ));
    assert!(!rule.covers(
        "AWS::SNS::Topic",
        &Difference {
            direction: Direction::Missing,
            identity: "prod-system-42"
        }
    ));
    assert_eq!(rule.reason, "emulator owns system topics");
    Ok(())
}

#[test]
fn given_identity_specific_explanation_when_parsed_then_rejected() -> TestResult {
    let error = rules("AWS::SNS::Topic\tmissing_from_sweep:one-id\treason")
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control rule must end in one wildcard, not name one identity"
    );
    Ok(())
}

#[test]
fn given_exact_and_arn_alias_collision_when_compared_then_exact_identity_has_priority() -> TestResult
{
    let listed = ids(&["topic"]);
    let stored = ids(&["arn:aws:sns:us-east-1:123456789012:topic", "topic"]);
    assert_eq!(
        compare(&listed, &stored)?,
        Comparison {
            matched: 1,
            differences: vec![Difference {
                direction: Direction::Extra,
                identity: "arn:aws:sns:us-east-1:123456789012:topic"
            },]
        }
    );
    Ok(())
}

#[test]
fn given_invalid_rule_columns_when_parsed_then_rejected_with_exact_reason() -> TestResult {
    let columns_error = "Cloud Control explanation must have type, rule, reason columns";
    let cases = [
        (
            "AWS::SNS::Topic\tmissing_from_sweep:*\treason",
            "Cloud Control rule must end in one wildcard, not name one identity",
        ),
        (
            "AWS::SNS::Topic\tmissing_from_sweep:system-*\t",
            columns_error,
        ),
        (
            "AWS::SNS::Topic\tmissing_from_sweep:system-*",
            columns_error,
        ),
        (
            "AWS::SNS::Topic\tmissing_from_sweep:system-*\treason\textra",
            columns_error,
        ),
    ];
    cases
        .into_iter()
        .try_for_each(|(text, expected)| -> TestResult {
            let error = rules(text).err().ok_or("expected an error")?;
            assert_eq!(error.to_string(), expected);
            Ok(())
        })
}

#[test]
fn given_invalid_rule_fields_when_parsed_then_exact_errors_are_returned() -> TestResult {
    let columns = "Cloud Control explanation must have type, rule, reason columns";
    let cases = [
        (
            "AWS::SNS::Topic\tunknown:prefix*\treason",
            "unknown Cloud Control difference direction",
        ),
        ("SNS::Topic\tmissing_from_sweep:prefix*\treason", columns),
        (
            "AWS::SNS::Topic\tmissing_from_sweep:pre**\treason",
            "Cloud Control rule must end in one wildcard, not name one identity",
        ),
        ("AWS::SNS::Topic\tmissing_from_sweep:prefix*\t   ", columns),
        (
            "# header\nAWS::SNS::Topic\tmissing_from_sweep:prefix*\treason\n# misplaced\n",
            "Cloud Control explanation comment after first rule at line 3",
        ),
    ];
    cases
        .into_iter()
        .try_for_each(|(text, expected)| -> TestResult {
            let error = rules(text).err().ok_or("expected rule error")?;
            assert_eq!(error.to_string(), expected);
            Ok(())
        })
}

#[test]
fn given_plain_identifier_when_arn_resource_called_then_none() -> TestResult {
    assert_eq!(arn_resource("my-topic"), None);
    assert_eq!(arn_resource("system-42"), None);
    Ok(())
}

#[test]
fn given_arn_when_arn_resource_called_then_resource_part_extracted() -> TestResult {
    assert_eq!(
        arn_resource("arn:aws:sns:us-east-1:123456789012:my-topic"),
        Some("my-topic")
    );
    assert_eq!(arn_resource("arn:aws:s3:::my-bucket"), Some("my-bucket"));
    Ok(())
}

#[test]
fn given_duplicate_rule_same_type_direction_prefix_when_parsed_then_rejected() -> TestResult {
    let text = "# type\trule\treason\nAWS::SNS::Topic\tmissing_from_sweep:system-*\tfirst\nAWS::SNS::Topic\tmissing_from_sweep:system-*\tsecond\n";
    let error = rules(text).err().ok_or("expected duplicate rule error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control duplicate explanation rule at line 3"
    );
    Ok(())
}

#[test]
fn given_rules_same_type_different_prefixes_when_parsed_then_accepted() -> TestResult {
    let text = "# type\trule\treason\nAWS::SNS::Topic\tmissing_from_sweep:system-*\tsystem topics\nAWS::SNS::Topic\tmissing_from_sweep:customer-*\tcustomer topics\n";
    let rules = rules(text)?;
    assert_eq!(rules.len(), 2);
    Ok(())
}

#[test]
fn given_header_only_when_parsed_then_zero_rules() -> TestResult {
    let text = "# type\trule\treason\n";
    let rules = rules(text)?;
    assert_eq!(rules.len(), 0);
    Ok(())
}

#[test]
fn given_tracked_differences_input_when_read_then_no_explanation_rows_are_present() -> TestResult {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../catalog/cloudcontrol-differences.tsv");
    let text = std::fs::read_to_string(path)?;
    assert_eq!(rules(&text)?.len(), 0);
    Ok(())
}

#[test]
fn given_typed_arn_resources_when_identifiers_omit_type_then_first_separator_is_removed()
-> TestResult {
    let listed = ids(&["handler", "vol-1"]);
    let stored = ids(&[
        "arn:aws:lambda:us-east-1:123456789012:function:handler",
        "arn:aws:ec2:us-east-1:123456789012:volume/vol-1",
    ]);
    assert_eq!(
        compare(&listed, &stored)?,
        Comparison {
            matched: 2,
            differences: vec![]
        }
    );
    Ok(())
}

#[test]
fn given_role_path_when_only_basename_is_listed_then_path_remains_a_difference() -> TestResult {
    let listed = ids(&["worker"]);
    let stored = ids(&["arn:aws:iam::123456789012:role/service-role/worker"]);
    assert_eq!(
        compare(&listed, &stored)?,
        Comparison {
            matched: 0,
            differences: vec![
                Difference {
                    direction: Direction::Missing,
                    identity: "worker"
                },
                Difference {
                    direction: Direction::Extra,
                    identity: "arn:aws:iam::123456789012:role/service-role/worker"
                },
            ],
        }
    );
    Ok(())
}
