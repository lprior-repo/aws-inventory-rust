use super::{
    SEEDED,
    fixture::{Fixture, execute, topic_report},
    server::TestResult,
};

#[tokio::test]
async fn given_two_stored_arn_candidates_when_command_runs_then_ambiguity_names_both() -> TestResult
{
    let function = "arn:aws:sns:us-east-1:123456789012:function:topic";
    let role = "arn:aws:sns:us-east-1:123456789012:role/topic";
    let fixture = Fixture::new(&[function, role], "list", SEEDED).await?;
    std::fs::write(&fixture.report, "previous report\n")?;
    let (output, requests) = execute(&fixture, &[(200, super::TOPIC)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr)?,
        format!(
            "Cloud Control AWS::SNS::Topic: ambiguous identifier \"topic\": [\"{function}\", \"{role}\"]\n"
        )
    );
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(
        std::fs::read_to_string(&fixture.report)?,
        "previous report\n"
    );
    assert_eq!(requests.len(), 1);
    Ok(())
}

async fn delimited_difference(identifier: &str, stored: &str) -> TestResult {
    let fixture = Fixture::new(&[stored], "list", SEEDED).await?;
    let body = serde_json::json!({"ResourceDescriptions":[{"Identifier":identifier}]}).to_string();
    let (output, requests) = execute(&fixture, &[(200, body.as_str())], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        topic_report(&output.stdout)?,
        format!(
            "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t1\nAWS::SNS::Topic\tmissing_from_sweep\t{identifier}\tunexplained\t\nAWS::SNS::Topic\textra_in_sweep\t{stored}\tunexplained\t\n"
        )
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_colon_in_listed_identifier_when_only_suffix_matches_then_it_stays_a_difference()
-> TestResult {
    delimited_difference(
        "handler:alias",
        "arn:aws:lambda:us-east-1:123456789012:function:handler:alias",
    )
    .await
}

#[tokio::test]
async fn given_slash_in_listed_identifier_when_only_suffix_matches_then_it_stays_a_difference()
-> TestResult {
    delimited_difference(
        "service-role/worker",
        "arn:aws:iam::123456789012:role/service-role/worker",
    )
    .await
}

#[tokio::test]
async fn given_role_path_when_only_basename_is_listed_then_cli_reports_unexplained_pair()
-> TestResult {
    delimited_difference(
        "worker",
        "arn:aws:iam::123456789012:role/service-role/worker",
    )
    .await
}
