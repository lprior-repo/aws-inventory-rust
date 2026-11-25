use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_missing_document_when_summarized_then_no_statements_are_emitted() -> TestResult {
    assert_eq!(summarize(None)?, Vec::<Value>::new());
    Ok(())
}

#[test]
fn given_encoded_document_when_summarized_then_plus_is_preserved() -> TestResult {
    let values = summarize(Some(
        "%7B%22Statement%22%3A%7B%22Effect%22%3A%22Allow%22%2C%22Principal%22%3A%7B%22Service%22%3A%22a+b.example%22%7D%2C%22Action%22%3A%22sts%3AAssumeRole%22%7D%7D",
    ))?;
    assert_eq!(
        values
            .first()
            .and_then(|value| value.pointer("/principals/Service/0")),
        Some(&json!("a+b.example"))
    );
    Ok(())
}

#[test]
fn given_condition_values_when_summarized_then_only_operators_are_retained() -> TestResult {
    let document = json!({"Statement": {"Effect": "Allow", "Principal": "*",
        "Action": "sts:AssumeRole", "Condition": {"StringEquals": {"secret": "NEVER_PERSIST"}}}});
    assert_eq!(
        summarize(Some(&document.to_string()))?,
        vec![json!({
            "effect": "Allow", "actions": ["sts:AssumeRole"], "principals": {"AWS": ["*"]},
            "has_not_principal": false, "has_not_action": false,
            "condition_operators": ["StringEquals"], "condition_values_omitted": true
        })]
    );
    Ok(())
}

#[test]
fn given_oversized_document_when_summarized_then_it_is_rejected() {
    let document = "a".repeat(MAX_DOCUMENT_BYTES.saturating_add(1));
    assert!(matches!(
        summarize(Some(&document)),
        Err(TrustSummaryError::DocumentTooLarge)
    ));
}

#[test]
fn given_invalid_encoding_when_summarized_then_it_is_rejected() {
    assert!(matches!(
        summarize(Some("%FF")),
        Err(TrustSummaryError::Utf8)
    ));
    assert!(matches!(summarize(Some("{")), Err(TrustSummaryError::Json)));
}

#[test]
fn given_no_statement_when_summarized_then_it_is_rejected() {
    assert!(matches!(
        summarize(Some("{}")),
        Err(TrustSummaryError::MissingStatement)
    ));
}

#[test]
fn given_statement_limit_when_summarized_then_only_the_bound_is_accepted() -> TestResult {
    let document = json!({"Statement": vec![json!({}); MAX_STATEMENTS]});
    assert_eq!(
        summarize(Some(&document.to_string()))?.len(),
        MAX_STATEMENTS
    );
    let document = json!({"Statement": vec![json!({}); MAX_STATEMENTS.saturating_add(1)]});
    assert!(matches!(
        summarize(Some(&document.to_string())),
        Err(TrustSummaryError::Statements)
    ));
    Ok(())
}

#[test]
fn given_invalid_statement_shape_when_summarized_then_it_is_rejected() {
    assert!(matches!(
        summarize(Some(r#"{"Statement":true}"#)),
        Err(TrustSummaryError::Statements)
    ));
    assert!(matches!(
        summarize(Some(r#"{"Statement":[false]}"#)),
        Err(TrustSummaryError::Statement)
    ));
}

#[test]
fn given_total_principal_limit_when_summarized_then_it_spans_all_statements() -> TestResult {
    let statement = json!({"Principal": {"AWS": vec!["123456789012"; 128]}});
    let document = json!({"Statement": [statement.clone(), statement.clone()]});
    assert_eq!(summarize(Some(&document.to_string()))?.len(), 2);
    let document = json!({"Statement": [statement.clone(), statement, {"Principal": "*"}]});
    assert!(matches!(
        summarize(Some(&document.to_string())),
        Err(TrustSummaryError::Principals)
    ));
    Ok(())
}

#[test]
fn given_invalid_references_when_summarized_then_they_are_rejected() {
    let document = json!({"Statement": {"Action": vec!["a"; 65]}});
    assert!(matches!(
        summarize(Some(&document.to_string())),
        Err(TrustSummaryError::Reference)
    ));
    assert!(matches!(
        summarize(Some(r#"{"Statement":{"Action":[false]}}"#)),
        Err(TrustSummaryError::Reference)
    ));
    assert!(matches!(
        summarize(Some(r#"{"Statement":{"Action":""}}"#)),
        Err(TrustSummaryError::Validation(_))
    ));
}

#[test]
fn given_invalid_principal_when_summarized_then_it_is_rejected() {
    assert!(matches!(
        summarize(Some(r#"{"Statement":{"Principal":true}}"#)),
        Err(TrustSummaryError::Principal)
    ));
}

#[test]
fn given_no_principal_when_summarized_then_negative_flags_are_preserved() -> TestResult {
    let document = r#"{"Statement":{"Effect":"Deny","NotPrincipal":"*","NotAction":"x"}}"#;
    assert_eq!(
        summarize(Some(document))?,
        vec![json!({
            "effect": "Deny", "actions": [], "principals": {}, "has_not_principal": true,
            "has_not_action": true, "condition_operators": [], "condition_values_omitted": false
        })]
    );
    Ok(())
}
