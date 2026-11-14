use super::{TestResult, model, paginated, replace};
use catalog_tool::plans::{
    Error, LargestPageSize, Model, Pagination, PaginationReason, PaginatorError, Vocabulary,
    pagination,
};
use serde_json::{Value, json};

#[test]
fn service_defaults_fill_missing_fields_and_operation_values_win() -> TestResult {
    // Given conflicting defaults, when merged, then the operation wins and dotted output survives.
    let value = paginated();
    let model = Model::parse(&value, "alpha")?;
    let mut vocabulary = Vocabulary::default();
    vocabulary.record(&model)?;
    let plan = pagination(&model, "n#ListRoots", &vocabulary)?;
    assert_eq!(
        plan,
        Pagination::Modeled {
            input_token: "Next",
            output_token: "Envelope.Token",
            items: Some("Envelope.Items"),
            page_size: Some("Limit")
        }
    );
    assert_eq!(
        plan.largest_page_size(&model, "n#ListRoots")?,
        Some(LargestPageSize::Known(90))
    );
    Ok(())
}

#[test]
fn missing_tokens_name_service_and_operation() -> TestResult {
    // Given an incomplete merged trait, when vocabulary is collected, then context is retained.
    let mut value = paginated();
    replace(&mut value, "/shapes/n#Service/traits", json!({}))?;
    let model = Model::parse(&value, "missing")?;
    let error = Vocabulary::default().record(&model);
    assert!(
        matches!(error, Err(Error::InvalidPaginator { service, operation, kind: PaginatorError::MissingToken })
        if service == "missing" && operation == "ListRoots")
    );
    Ok(())
}

#[test]
fn missing_token_members_and_malformed_paths_are_typed_errors() -> TestResult {
    // Given invalid merged output/input paths, none is silently treated as unpaginated.
    [
        ("Absent", "Envelope.Token", PaginatorError::InputMember),
        ("Next", "Missing.Token", PaginatorError::OutputMember),
        ("Next", "Envelope..Token", PaginatorError::Path),
    ].into_iter().try_for_each(|(input_token, output_token, kind)| {
        let mut value = paginated();
        replace(&mut value, "/shapes/n#ListRoots/traits/smithy.api#paginated", json!({"inputToken": input_token, "outputToken": output_token}))?;
        let model = Model::parse(&value, "svc")?;
        assert!(matches!(Vocabulary::default().record(&model), Err(Error::InvalidPaginator { kind: found, .. }) if found == kind));
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

fn suspected(input: Value, output: Value, boolean: &str) -> Value {
    model(json!({
        "n#ListUnknown":{"type":"operation","input":{"target":"n#I"},"output":{"target":"n#O"}},
        "n#I":{"type":"structure","members":input},
        "n#O":{"type":"structure","members":output},
        "n#Flag":{"type":boolean}
    }))
}

#[test]
fn cross_service_vocabulary_and_reason_precedence_are_exact() -> TestResult {
    // Given vocabulary from another service, both token sides outrank one side and truncation.
    let source = paginated();
    let mut vocab = Vocabulary::default();
    vocab.record(&Model::parse(&source, "other")?)?;
    let token = json!({"target":"smithy.api#String"});
    let flag = json!({"target":"n#Flag"});
    [
        (
            json!({"Next":token}),
            json!({"Token":token,"IsTruncated":flag}),
            PaginationReason::TokenBothSides,
        ),
        (
            json!({"Next":token}),
            json!({"IsTruncated":flag}),
            PaginationReason::TokenInputOnly,
        ),
        (
            json!({}),
            json!({"Token":token,"HasMore":flag}),
            PaginationReason::TokenOutputOnly,
        ),
        (
            json!({}),
            json!({"IsTruncated":flag}),
            PaginationReason::TruncationFlag,
        ),
    ]
    .into_iter()
    .try_for_each(|(input, output, reason)| {
        let value = suspected(input, output, "boolean");
        let model = Model::parse(&value, "local")?;
        assert_eq!(
            pagination(&model, "n#ListUnknown", &vocab)?,
            Pagination::NeedsOverlay(reason)
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn nonboolean_more_member_and_absent_vocabulary_are_unpaginated() -> TestResult {
    // Given lookalike names without matching types/vocabulary, no token heuristic is invented.
    let value = suspected(
        json!({"Next":{"target":"smithy.api#String"}}),
        json!({"IsTruncated":{"target":"n#Flag"}}),
        "string",
    );
    let model = Model::parse(&value, "svc")?;
    assert_eq!(
        pagination(&model, "n#ListUnknown", &Vocabulary::default())?,
        Pagination::Unpaginated
    );
    Ok(())
}

#[test]
fn operation_output_overrides_default_and_default_input_fills_its_absence() -> TestResult {
    // Given only an operation output override, merging fills input but preserves that output.
    let mut value = paginated();
    replace(
        &mut value,
        "/shapes/n#Input/members",
        json!({
        "Cursor":{"target":"smithy.api#String"}, "Limit":{"target":"n#Size"}}),
    )?;
    replace(
        &mut value,
        "/shapes/n#ListRoots/traits/smithy.api#paginated",
        json!({"outputToken":"Envelope.Override"}),
    )?;
    let model = Model::parse(&value, "svc")?;
    assert_eq!(
        pagination(&model, "n#ListRoots", &Vocabulary::default())?,
        Pagination::Modeled {
            input_token: "Cursor",
            output_token: "Envelope.Override",
            items: Some("Envelope.Items"),
            page_size: Some("Limit")
        }
    );
    Ok(())
}

#[test]
fn non_list_modeled_operations_still_supply_cross_service_vocabulary() -> TestResult {
    // Given a modeled Get paginator, when vocabulary is collected, then it protects another service.
    let mut donor = paginated();
    let shapes = donor
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    let op = shapes.remove("n#ListRoots").ok_or("operation missing")?;
    shapes.insert("n#GetSeed".to_owned(), op);
    let mut vocab = Vocabulary::default();
    vocab.record(&Model::parse(&donor, "donor")?)?;
    let value = suspected(
        json!({}),
        json!({"Token":{"target":"smithy.api#String"}}),
        "string",
    );
    let model = Model::parse(&value, "recipient")?;
    assert_eq!(
        pagination(&model, "n#ListUnknown", &vocab)?,
        Pagination::NeedsOverlay(PaginationReason::TokenOutputOnly)
    );
    Ok(())
}

#[test]
fn token_paths_accept_the_depth_bound_and_reject_one_beyond() -> TestResult {
    // Given the named path-depth boundary, when collecting, only the oversized path fails.
    [32_usize, 33].into_iter().try_for_each(|depth| {
        let mut value = paginated();
        let path = vec!["Envelope"; depth].join(".");
        replace(
            &mut value,
            "/shapes/n#Service/traits/smithy.api#paginated/outputToken",
            json!(path),
        )?;
        let model = Model::parse(&value, "svc")?;
        let result = Vocabulary::default().record(&model);
        if depth == 32 {
            result?;
        } else {
            assert!(matches!(
                result,
                Err(Error::InvalidPaginator {
                    kind: PaginatorError::Path,
                    ..
                })
            ));
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
