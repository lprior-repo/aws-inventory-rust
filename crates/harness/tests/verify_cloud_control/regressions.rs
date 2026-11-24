use super::{
    SEEDED, TOPIC,
    fixture::{Fixture, execute, topic_report},
    server::TestResult,
    store,
};
use futures_util::TryStreamExt;

#[tokio::test]
async fn given_throttling_on_second_page_when_command_runs_then_typed_service_error_fails_closed()
-> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    std::fs::write(&fixture.report, "previous report\n")?;
    let differences = std::fs::read(&fixture.differences)?;
    let (output, requests) = execute(
        &fixture,
        &[
            (
                200,
                r#"{"ResourceDescriptions":[{"Identifier":"topic"}],"NextToken":"next"}"#,
            ),
            (
                400,
                r#"{"__type":"ThrottlingException","message":"slow down"}"#,
            ),
        ],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control ListResources AWS::SNS::Topic: service error\n"
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(
        std::fs::read_to_string(&fixture.report)?,
        "previous report\n"
    );
    assert_eq!(std::fs::read(&fixture.differences)?, differences);
    assert_eq!(requests.len(), 2);
    let second = requests.get(1).ok_or("missing second-page request")?;
    assert!(second.contains(r#""NextToken":"next""#));
    Ok(())
}

#[tokio::test]
async fn given_missing_differences_input_when_command_runs_then_error_names_path_without_requests()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    std::fs::remove_file(&fixture.differences)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "Cloud Control differences input not found: {}\n",
            fixture.differences.display()
        )
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    assert!(!fixture.differences.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_unknown_seed_kinds_when_command_runs_then_sorted_error_precedes_requests()
-> TestResult {
    let fixture = Fixture::new(
        &[],
        "list",
        concat!(
            "resource\tAWS::Typo::Zed\t123456789012\tus-east-1\tz\n",
            "resource\tAWS::Typo::Alpha\t123456789012\tus-east-1\ta\n",
        ),
    )
    .await?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control kind not in catalog: [\"AWS::Typo::Alpha\", \"AWS::Typo::Zed\"]\n"
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    assert_eq!(
        std::fs::read_to_string(&fixture.differences)?,
        "# type\trule\treason\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_cfn_kinds_without_targets_when_command_runs_then_fails_before_network() -> TestResult
{
    let fixture = Fixture::new(
        &[],
        "list",
        concat!(
            "resource\tAWS::Glue::Table\t123456789012\tus-east-1\ttable\n",
            "resource\tAWS::CloudWatch::Alarm\t123456789012\tus-east-1\talarm\n",
            "resource\tAWS::ControlTower::LandingZone\t123456789012\tus-east-1\tzone\n",
        ),
    )
    .await?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control seeded kinds have no mapped target: [\"AWS::CloudWatch::Alarm\", \"AWS::ControlTower::LandingZone\", \"AWS::Glue::Table\"]\n"
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert!(!fixture.report.try_exists()?);
    assert_eq!(
        std::fs::read_to_string(&fixture.differences)?,
        "# type\trule\treason\n"
    );
    assert_eq!(requests, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn given_type_not_found_when_command_runs_then_type_is_not_listable() -> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let (output, requests) = execute(
        &fixture,
        &[(
            400,
            r#"{"__type":"TypeNotFoundException","message":"unsupported type"}"#,
        )],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tnot listable\n"
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    Ok(())
}

async fn reject_service_error(code: &str) -> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let body = serde_json::json!({"__type":code,"message":"denied"}).to_string();
    let (output, requests) = execute(&fixture, &[(400, body.as_str())], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control ListResources AWS::SNS::Topic: service error\n"
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert!(!fixture.report.try_exists()?);
    assert_eq!(
        std::fs::read_to_string(&fixture.differences)?,
        "# type\trule\treason\n"
    );
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_access_denied_when_command_runs_then_no_false_not_listable_report() -> TestResult {
    reject_service_error("AccessDeniedException").await
}

#[tokio::test]
async fn given_generic_not_implemented_when_command_runs_then_fails_closed() -> TestResult {
    reject_service_error("NotImplemented").await
}

#[tokio::test]
async fn given_generic_not_supported_when_command_runs_then_fails_closed() -> TestResult {
    reject_service_error("NotSupported").await
}

#[tokio::test]
async fn given_seeded_kind_without_cfn_type_when_command_runs_then_reports_exclusion() -> TestResult
{
    let fixture = Fixture::new(
        &[],
        "unsupported",
        "resource\tAWS::ApiOnly::SSMActivation\t123456789012\tus-east-1\tactivation\n",
    )
    .await?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout)?;
    assert!(text.starts_with("# no CloudFormation type\tcount\t1\nAWS::ApiOnly::SSMActivation\tno CloudFormation type\n# Cloud Control comparisons\n"));
    assert_eq!(std::fs::read_to_string(&fixture.report)?, text);
    assert_eq!(requests, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn given_seed_directory_without_oracle_when_difference_found_then_gate_fails() -> TestResult {
    let mut fixture = Fixture::new(&[], "list", "").await?;
    fixture.oracle = None;
    let seeds = fixture.work.path().join("seeds");
    std::fs::create_dir(&seeds)?;
    std::fs::write(
        seeds.join("sns.toml"),
        "format = 1\nservice = \"sns\"\n[[step]]\nid = \"topic\"\noperation = \"CreateTopic\"\nkind = \"AWS::SNS::Topic\"\nidentity = \"TopicArn\"\n[step.input]\nName = \"seeded-topic\"\n",
    )?;
    let (output, _) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t0\nAWS::SNS::Topic\tmissing_from_sweep\ttopic\tunexplained\t\n"
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    Ok(())
}

#[tokio::test]
async fn given_empty_next_token_when_command_runs_then_pagination_finishes() -> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let (output, requests) = execute(
        &fixture,
        &[(200, r#"{"ResourceDescriptions":[],"NextToken":""}"#)],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_full_page_when_command_runs_then_all_descriptions_are_accepted() -> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let descriptions = vec![serde_json::json!({"Identifier":"topic"}); 100];
    let body = serde_json::json!({"ResourceDescriptions":descriptions}).to_string();
    let (output, requests) = execute(&fixture, &[(200, body.as_str())], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_oversized_page_when_command_runs_then_partial_comparison_is_rejected() -> TestResult
{
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let descriptions = vec![serde_json::json!({"Identifier":"topic"}); 101];
    let body = serde_json::json!({"ResourceDescriptions":descriptions}).to_string();
    let (output, requests) = execute(&fixture, &[(200, body.as_str())], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control page identity bound exceeded\n"
    );
    assert!(!fixture.report.try_exists()?);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_report_directory_when_command_runs_then_rejected_without_requests_or_input_changes()
-> TestResult {
    let mut fixture = Fixture::new(&[], "list", SEEDED).await?;
    fixture.report = fixture.work.path().join("report-directory");
    std::fs::create_dir(&fixture.report)?;
    let rules = concat!(
        "# handwritten\n",
        "AWS::SNS::Topic\tmissing_from_sweep:z*\tlast\n",
        "AWS::SNS::Topic\tmissing_from_sweep:a*\tfirst\n",
        "AWS::SNS::Topic\tmissing_from_sweep:a*\tfirst\n",
    );
    std::fs::write(&fixture.differences, rules)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "Cloud Control report path is a directory: {}\n",
            fixture.report.display()
        )
    );
    assert_eq!(std::fs::read_to_string(&fixture.differences)?, rules);
    assert!(fixture.report.is_dir());
    assert_eq!(requests, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn given_unsupported_after_first_page_when_listing_then_fails_instead_of_dropping_identities()
-> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let codes = [
        "UnsupportedAction",
        "UnsupportedActionException",
        "TypeNotFoundException",
    ];
    let fixture = &fixture;
    futures_util::stream::iter(codes.into_iter().map(Ok::<_, Box<dyn std::error::Error>>))
        .try_for_each(|code| async move {
            let body = serde_json::json!({"__type":code,"message":"unsupported"}).to_string();
            let replies = [
                (
                    200,
                    r#"{"ResourceDescriptions":[{"Identifier":"topic"}],"NextToken":"second"}"#,
                ),
                (400, body.as_str()),
            ];
            let (output, requests) = execute(fixture, &replies, None).await?;
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(
                String::from_utf8(output.stderr)?,
                "Cloud Control ListResources AWS::SNS::Topic: service error\n"
            );
            assert!(!fixture.report.try_exists()?);
            assert_eq!(requests.len(), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn given_empty_primary_identifier_when_command_runs_then_invalid_evidence_is_rejected()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let (output, _) = execute(
        &fixture,
        &[(200, r#"{"ResourceDescriptions":[{"Identifier":""}]}"#)],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control resource lacks primary identifier\n"
    );
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_comment_after_rule_when_command_runs_then_line_number_is_reported_without_rewriting()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let text = "# leading\nAWS::SNS::Topic\tmissing_from_sweep:topic*\treason\n# misplaced\n";
    std::fs::write(&fixture.differences, text)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control explanation comment after first rule at line 3\n"
    );
    assert_eq!(std::fs::read_to_string(&fixture.differences)?, text);
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_differences_file_over_one_mib_when_command_runs_then_byte_bound_error() -> TestResult
{
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let oversized = fixture.work.path().join("oversized.tsv");
    let bytes = vec![0u8; 1_048_577];
    std::fs::write(&oversized, &bytes)?;
    std::fs::rename(&oversized, &fixture.differences)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control input byte bound exceeded\n"
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_directory_as_differences_path_when_command_runs_then_rejected() -> TestResult {
    let mut fixture = Fixture::new(&[], "list", SEEDED).await?;
    fixture.differences = fixture.work.path().join("differences-directory");
    std::fs::create_dir(&fixture.differences)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "Cloud Control differences input is a directory: {}\n",
            fixture.differences.display()
        )
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_unexpected_request_when_command_runs_then_fixture_rejects_it() -> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let result = execute(&fixture, &[], None).await;
    let error = result.err().ok_or("expected unexpected-request error")?;
    assert_eq!(error.to_string(), "unexpected Cloud Control request");
    Ok(())
}

#[tokio::test]
async fn given_unexpected_second_page_when_command_runs_then_fixture_rejects_it() -> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let replies = [(
        200,
        r#"{"ResourceDescriptions":[],"NextToken":"unexpected"}"#,
    )];
    let result = execute(&fixture, &replies, None).await;
    let error = result.err().ok_or("expected unexpected-request error")?;
    assert_eq!(error.to_string(), "unexpected Cloud Control request");
    Ok(())
}

#[tokio::test]
async fn given_unexpected_connection_when_server_accepts_then_failure_is_immediate() -> TestResult {
    let server = super::server::Server::new().await?;
    let endpoint = server.endpoint()?;
    let address = endpoint
        .strip_prefix("https://")
        .ok_or("missing HTTPS prefix")?;
    let (_done, finished) = tokio::sync::oneshot::channel();
    let connection = tokio::net::TcpStream::connect(address).await?;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        server.serve(&[], finished),
    )
    .await?;
    drop(connection);
    let error = result.err().ok_or("expected unexpected-request error")?;
    assert_eq!(error.to_string(), "unexpected Cloud Control request");
    Ok(())
}

#[tokio::test]
async fn given_mapped_type_absent_from_universal_when_command_runs_then_error_names_type()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let path = fixture.work.path().join("universal.report");
    let mut universal: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    universal
        .get_mut("resource_types")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or("missing resource types")?
        .retain(|entry| {
            entry.get("type_name").and_then(serde_json::Value::as_str) != Some("AWS::SNS::Topic")
        });
    std::fs::write(&path, serde_json::to_vec(&universal)?)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control type missing from universal catalog: AWS::SNS::Topic\n"
    );
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

async fn regional_fixture() -> TestResult<Fixture> {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    store::append(
        &fixture.database,
        &[
            store::resource("123456789012", "us-west-2", "AWS::SNS::Topic", "topic")?,
            store::resource("123456789012", "us-west-2", "AWS::SNS::Topic", "west-only")?,
        ],
    )
    .await?;
    Ok(fixture)
}

#[tokio::test]
async fn given_same_identifier_in_two_regions_when_command_runs_then_other_region_is_excluded()
-> TestResult {
    let fixture = regional_fixture().await?;
    let (output, requests) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stderr)?, "");
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_multiple_accounts_when_command_runs_then_explicit_account_resolves_ambiguity()
-> TestResult {
    let mut fixture = regional_fixture().await?;
    store::append(
        &fixture.database,
        &[store::resource(
            "111111111111",
            "us-east-1",
            "AWS::SNS::Topic",
            "foreign-topic",
        )?],
    )
    .await?;
    let (output, requests) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control kind AWS::SNS::Topic has multiple inventory accounts: [\"111111111111\", \"123456789012\"]; specify --account\n"
    );
    assert_eq!(requests.len(), 1);
    assert!(!fixture.report.try_exists()?);
    fixture.account = Some("123456789012".into());
    let (output, requests) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stderr)?, "");
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    Ok(())
}
