use super::{
    SEEDED,
    fixture::{Fixture, execute, topic_report},
    server::TestResult,
};

const RESPONSE_BYTES: usize = 1_048_576;
const IDENTIFIER_BYTES: usize = 4096;

fn padded_response(bytes: usize) -> TestResult<String> {
    let prefix = r#"{"ResourceDescriptions":[{"Identifier":"topic","Properties":""#;
    let suffix = r#""}]}"#;
    let padding = bytes
        .checked_sub(prefix.len())
        .and_then(|remaining| remaining.checked_sub(suffix.len()))
        .ok_or("response bound is smaller than the JSON envelope")?;
    Ok(format!("{prefix}{}{suffix}", "x".repeat(padding)))
}

async fn reject_bound(body: &str, message: &str) -> TestResult {
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    std::fs::write(&fixture.report, "previous report\n")?;
    let differences = std::fs::read(&fixture.differences)?;
    let (output, requests) = execute(&fixture, &[(200, body)], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8(output.stderr)?, message);
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(
        std::fs::read_to_string(&fixture.report)?,
        "previous report\n"
    );
    assert_eq!(std::fs::read(&fixture.differences)?, differences);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_response_at_byte_bound_when_command_runs_then_json_is_compared() -> TestResult {
    let fixture = Fixture::new(&["topic"], "list", SEEDED).await?;
    let body = padded_response(RESPONSE_BYTES)?;
    assert_eq!(body.len(), RESPONSE_BYTES);
    let (output, requests) = execute(&fixture, &[(200, body.as_str())], None).await?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        topic_report(&output.stdout)?,
        "AWS::SNS::Topic\tmatched\t1\tmissing_from_sweep\t0\textra_in_sweep\t0\n"
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_response_one_byte_over_bound_when_command_runs_then_typed_bound_error() -> TestResult
{
    let body = padded_response(RESPONSE_BYTES.saturating_add(1))?;
    reject_bound(
        &body,
        "Cloud Control AWS::SNS::Topic page 1 response exceeds 1048576 bytes\n",
    )
    .await
}

#[tokio::test]
async fn given_oversized_invalid_json_when_command_runs_then_byte_bound_precedes_json_parsing()
-> TestResult {
    reject_bound(
        &"x".repeat(RESPONSE_BYTES.saturating_add(1)),
        "Cloud Control AWS::SNS::Topic page 1 response exceeds 1048576 bytes\n",
    )
    .await
}

#[tokio::test]
async fn given_multibyte_identifier_at_byte_bound_when_command_runs_then_difference_is_reported()
-> TestResult {
    let identifier = "é".repeat(IDENTIFIER_BYTES / 2);
    assert_eq!(identifier.len(), IDENTIFIER_BYTES);
    let body = serde_json::json!({"ResourceDescriptions":[{"Identifier":identifier}]}).to_string();
    let fixture = Fixture::new(&[], "list", SEEDED).await?;
    let (output, requests) = execute(&fixture, &[(200, body.as_str())], None).await?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, Vec::<u8>::new());
    assert_eq!(
        topic_report(&output.stdout)?,
        format!(
            "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t0\nAWS::SNS::Topic\tmissing_from_sweep\t{identifier}\tunexplained\t\n"
        )
    );
    assert_eq!(std::fs::read(&fixture.report)?, output.stdout);
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_multibyte_identifier_one_byte_over_bound_when_command_runs_then_typed_bound_error()
-> TestResult {
    let identifier = format!("{}x", "é".repeat(IDENTIFIER_BYTES / 2));
    assert_eq!(identifier.len(), IDENTIFIER_BYTES.saturating_add(1));
    let body = serde_json::json!({"ResourceDescriptions":[{"Identifier":identifier}]}).to_string();
    reject_bound(
        &body,
        "Cloud Control AWS::SNS::Topic page 1 identifier exceeds 4096 bytes\n",
    )
    .await
}
