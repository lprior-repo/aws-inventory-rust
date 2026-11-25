#[path = "verify_cloud_control/bounds.rs"]
mod bounds;
#[path = "verify_cloud_control/fixture.rs"]
mod fixture;
#[path = "verify_cloud_control/matching.rs"]
mod matching;
#[path = "verify_cloud_control/postgres.rs"]
mod postgres;
#[path = "verify_cloud_control/regressions.rs"]
mod regressions;
#[path = "verify_cloud_control/scopes.rs"]
mod scopes;
#[path = "verify_cloud_control/server.rs"]
mod server;
#[path = "verify_cloud_control/store.rs"]
mod store;

use fixture::{Fixture, execute, topic_report};
use server::TestResult;

const SEEDED: &str = "resource\tAWS::SNS::Topic\t123456789012\tus-east-1\ttopic\n";
const TOPIC: &str = r#"{"ResourceDescriptions":[{"Identifier":"topic"}]}"#;

#[tokio::test]
async fn given_exactly_256_pages_when_command_runs_then_complete_comparison_succeeds() -> TestResult
{
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let pages: Vec<String> = (0..255)
        .map(|page| format!("{{\"ResourceDescriptions\":[],\"NextToken\":\"page-{page}\"}}"))
        .chain(std::iter::once(TOPIC.to_owned()))
        .collect();
    let replies: Vec<_> = pages.iter().map(|page| (200, page.as_str())).collect();
    let (output, requests) = execute(&fixture, &replies, None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stderr)?, "");
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(requests.len(), 256);
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    Ok(())
}
#[tokio::test]
async fn given_matching_arn_when_command_runs_then_success_and_read_only_inventory() -> TestResult {
    let fixture = Fixture::new(
        &["arn:aws:sns:us-east-1:123456789012:topic"],
        "list",
        SEEDED,
    )
    .await?;
    let before = std::fs::read(&fixture.database)?;
    let (output, requests) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(
        std::fs::read_to_string(&fixture.differences)?,
        "# type\trule\treason\n"
    );
    assert_eq!(std::fs::read(&fixture.database)?, before);
    assert_eq!(requests.len(), 1);
    let request = requests.first().ok_or("missing request")?;
    assert!(request.contains("CloudApiService.ListResources"));
    assert!(request.contains("\"MaxResults\":100"));
    Ok(())
}

#[tokio::test]
async fn given_missing_identity_when_command_runs_then_difference_report_survives_failure()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
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
async fn given_extra_identity_when_command_runs_then_extra_direction_is_reported() -> TestResult {
    let fixture = Fixture::new(&["topic", "matching"], "list", SEEDED).await?;
    let listed = r#"{"ResourceDescriptions":[{"Identifier":"matching"}]}"#;
    let (output, _) = execute(&fixture, &[(200, listed)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t1\nAWS::SNS::Topic\textra_in_sweep\ttopic\tunexplained\t\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_explanation_rule_when_command_runs_then_reason_is_preserved_and_success()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let rules = "AWS::SNS::Topic\tmissing_from_sweep:topic*\temulator-owned topics\n";
    std::fs::write(&fixture.differences, rules)?;
    let (output, _) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t0\nAWS::SNS::Topic\tmissing_from_sweep\ttopic\texplained\temulator-owned topics\n"
    );
    assert_eq!(std::fs::read_to_string(&fixture.differences)?, rules);
    let report = std::fs::read(&fixture.report)?;
    let differences = std::fs::read(&fixture.differences)?;
    let modified = std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(946_684_800))
        .ok_or("invalid fixed timestamp")?;
    std::fs::File::open(&fixture.differences)?.set_modified(modified)?;
    let (second, _) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(second.stdout, output.stdout);
    assert_eq!(std::fs::read(&fixture.report)?, report);
    assert_eq!(std::fs::read(&fixture.differences)?, differences);
    assert_eq!(
        std::fs::metadata(&fixture.differences)?.modified()?,
        modified
    );
    Ok(())
}

#[tokio::test]
async fn given_extra_in_sweep_rule_when_command_runs_then_explained_and_success() -> TestResult {
    let fixture = Fixture::new(&["topic", "matching"], "list", SEEDED).await?;
    let rules = "AWS::SNS::Topic\textra_in_sweep:topic*\temulator leftover topics\n";
    std::fs::write(&fixture.differences, rules)?;
    let listed = r#"{"ResourceDescriptions":[{"Identifier":"matching"}]}"#;
    let (output, _) = execute(&fixture, &[(200, listed)], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t1\nAWS::SNS::Topic\textra_in_sweep\ttopic\texplained\temulator leftover topics\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_wrong_direction_rule_when_command_runs_then_unexplained_and_fails() -> TestResult {
    let fixture = Fixture::new(&["topic", "matching"], "list", SEEDED).await?;
    let rules = "AWS::SNS::Topic\tmissing_from_sweep:topic*\temulator leftover topics\n";
    std::fs::write(&fixture.differences, rules)?;
    let listed = r#"{"ResourceDescriptions":[{"Identifier":"matching"}]}"#;
    let (output, _) = execute(&fixture, &[(200, listed)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t1\nAWS::SNS::Topic\textra_in_sweep\ttopic\tunexplained\t\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_unlistable_catalog_type_when_command_runs_then_no_api_call_or_difference()
-> TestResult {
    let fixture = Fixture::new(&["topic"], "unsupported", SEEDED).await?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tnot listable\n"
    );
    assert_eq!(requests, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn given_emulator_unsupported_type_when_command_runs_then_not_listable() -> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let (output, _) = execute(
        &fixture,
        &[(
            400,
            r#"{"__type":"UnsupportedActionException","message":"unsupported"}"#,
        )],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tnot listable\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_unsupported_action_on_first_page_when_command_runs_then_not_listable() -> TestResult
{
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let (output, _) = execute(
        &fixture,
        &[(
            400,
            r#"{"__type":"UnsupportedAction","message":"unsupported"}"#,
        )],
        None,
    )
    .await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tnot listable\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_unseeded_difference_when_command_runs_then_report_does_not_fail_seed_gate()
-> TestResult {
    let fixture = Fixture::new(&[], "list", "").await?;
    let (output, _) = execute(&fixture, &[(200, TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t0\nAWS::SNS::Topic\tmissing_from_sweep\ttopic\tunexplained\t\n"
    );
    Ok(())
}

#[tokio::test]
async fn given_two_pages_when_command_runs_then_all_identities_are_compared() -> TestResult {
    let fixture = Fixture::new(&["first", "second"], "list", SEEDED).await?;
    let replies = [
        (
            200,
            r#"{"ResourceDescriptions":[{"Identifier":"first"}],"NextToken":"page-two"}"#,
        ),
        (200, r#"{"ResourceDescriptions":[{"Identifier":"second"}]}"#),
    ];
    let (output, requests) = execute(&fixture, &replies, None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t2\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(requests.len(), 2);
    assert!(
        !requests
            .first()
            .ok_or("missing first request")?
            .contains("\"NextToken\"")
    );
    assert!(
        requests
            .get(1)
            .ok_or("missing second request")?
            .contains("\"NextToken\":\"page-two\"")
    );
    Ok(())
}

#[tokio::test]
async fn given_repeated_page_token_when_command_runs_then_fails_instead_of_comparing_partial_list()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let reply = r#"{"ResourceDescriptions":[],"NextToken":"same"}"#;
    let (output, requests) = execute(&fixture, &[(200, reply), (200, reply)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control repeated pagination token\n"
    );
    assert_eq!(requests.len(), 2);
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_inventory_as_report_path_when_command_runs_then_rejected_before_network()
-> TestResult {
    let mut fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    fixture.report = fixture.database.clone();
    let before = std::fs::read(&fixture.database)?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control outputs must not replace inventory or input files\n"
    );
    assert_eq!(requests, Vec::<String>::new());
    assert_eq!(std::fs::read(&fixture.database)?, before);
    Ok(())
}

#[tokio::test]
async fn given_page_limit_with_continuation_when_command_runs_then_no_partial_success() -> TestResult
{
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let pages: Vec<String> = (0..256)
        .map(|page| format!("{{\"ResourceDescriptions\":[],\"NextToken\":\"page-{page}\"}}"))
        .collect();
    let replies: Vec<_> = pages.iter().map(|page| (200, page.as_str())).collect();
    let (output, requests) = execute(&fixture, &replies, None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control page bound exceeded\n"
    );
    assert_eq!(requests.len(), 256);
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_missing_primary_identifier_when_command_runs_then_rejects_invalid_evidence()
-> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let (output, _) = execute(&fixture, &[(200, r#"{"ResourceDescriptions":[{}]}"#)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control resource lacks primary identifier\n"
    );
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[tokio::test]
async fn given_identity_specific_rule_when_command_runs_then_rejects_before_network() -> TestResult
{
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    std::fs::write(
        &fixture.differences,
        "AWS::SNS::Topic\tmissing_from_sweep:topic\treason\n",
    )?;
    let (output, requests) = execute(&fixture, &[], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Cloud Control rule must end in one wildcard, not name one identity\n"
    );
    assert_eq!(requests, Vec::<String>::new());
    assert!(!fixture.report.try_exists()?);
    Ok(())
}

#[test]
fn given_missing_required_flags_when_command_parsed_then_names_missing_database() -> TestResult {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
        .args(["verify", "cloud-control"])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "verify cloud-control requires --database\n"
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    Ok(())
}
