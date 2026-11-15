use super::*;

#[test]
fn given_no_modeled_paginator_when_reviewed_tuple_compiles_then_all_components_survive() -> TestResult {
    let value = compile_fixture(MODEL, PAIRS)?;
    let tokens = tokens(&value)?;
    assert_eq!(tokens.len(), 3);
    let last = inventory_core::record::Record::encode(tokens.last().ok_or("optional component")?)?;
    assert!(matches!(last.root().get_field(FieldId::new(6)?)?, Some(ValueRef::Bool(true))));
    let first = inventory_core::record::Record::encode(tokens.first().ok_or("required component")?)?;
    assert!(matches!(first.root().get_field(FieldId::new(6)?)?, Some(ValueRef::Bool(false))));
    Ok(())
}

#[test]
fn given_modeled_first_pair_when_reviewed_tuple_extends_it_then_compilation_succeeds() -> TestResult {
    let model = MODEL.replace("\"smithy.api#readonly\":{}", "\"smithy.api#readonly\":{},\"smithy.api#paginated\":{\"inputToken\":\"StartName\",\"outputToken\":\"NextName\",\"items\":\"Items\"}");
    assert_eq!(tokens(&compile_fixture(&model, PAIRS)?)?.len(), 3);
    let wrong = [("StartName", "NextId", false), ("StartId", "NextName", false)];
    assert!(compile_fixture(&model, &wrong).is_err());
    Ok(())
}

#[test]
fn given_invalid_token_path_or_type_when_compiling_then_review_is_rejected() -> TestResult {
    for pairs in [
        vec![("Missing", "NextName", false)],
        vec![("StartName", "IsTruncated", false)],
        vec![("StartName", "Items", false)],
        vec![("StartName", "NextName", false), ("StartType", "NextName", false)],
    ] {
        assert!(compile_fixture(MODEL, &pairs).is_err());
    }
    Ok(())
}

#[test]
fn given_required_request_member_when_review_marks_component_optional_then_compilation_fails() -> TestResult {
    let model = MODEL.replace("\"smithy.api#httpQuery\":\"identifier\"", "\"smithy.api#httpQuery\":\"identifier\",\"smithy.api#required\":{}");
    assert!(compile_fixture(&model, PAIRS).is_err());
    Ok(())
}

#[test]
fn given_wrong_truncated_type_when_compiling_then_review_is_rejected() -> TestResult {
    let model = MODEL.replace("\"IsTruncated\":{\"target\":\"smithy.api#Boolean\"}",
        "\"IsTruncated\":{\"target\":\"smithy.api#String\"}");
    assert!(compile_fixture(&model, PAIRS).is_err());
    Ok(())
}

#[test]
fn given_all_optional_or_empty_typed_schema_when_reviewed_then_review_is_rejected() -> TestResult {
    // Non-boolean optional flags are unrepresentable in the typed Rust policy.
    for pairs in [vec![("StartName", "NextName", true)], vec![]] {
        assert!(compile_fixture(MODEL, &pairs).is_err());
    }
    Ok(())
}

#[test]
fn given_overlay_only_marker_when_compiling_then_single_pair_is_preserved() -> TestResult {
    assert_eq!(tokens(&compile_fixture(MODEL, &[("StartName", "NextName", false)])?)?.len(), 1);
    Ok(())
}

#[test]
fn given_unsupported_response_placement_when_compiling_then_review_is_rejected() -> TestResult {
    let model = MODEL.replace("\"NextName\":{\"target\":\"smithy.api#String\"}",
        "\"NextName\":{\"target\":\"smithy.api#String\",\"traits\":{\"smithy.api#httpQuery\":\"next\"}}");
    assert!(compile_fixture(&model, PAIRS).is_err());
    Ok(())
}

#[test]
fn given_one_through_eight_reviewed_components_when_compiling_then_every_pair_survives() -> TestResult {
    let extra_inputs = (4..=8).map(|id| format!("\"Start{id}\":{{\"target\":\"smithy.api#String\"}},")).collect::<String>();
    let extra_outputs = (4..=8).map(|id| format!("\"Next{id}\":{{\"target\":\"smithy.api#String\"}},")).collect::<String>();
    let model = MODEL.replace("\"StartName\":", &format!("{extra_inputs}\"StartName\":"))
        .replace("\"NextName\":", &format!("{extra_outputs}\"NextName\":"));
    let extra_names = (4..=8).flat_map(|id| [format!("Start{id}"), format!("Next{id}")]);
    let mut names = REGISTRY.lines().map(str::to_owned).chain(extra_names).collect::<Vec<_>>();
    names.sort();
    let registry = format!("{}\n", names.join("\n"));
    let paths = ["Name", "Type", "Id", "4", "5", "6", "7", "8"];
    for count in 1..=8 {
        let paths = paths.iter().take(count)
            .map(|suffix| (format!("Start{suffix}"), format!("Next{suffix}")))
            .collect::<Vec<_>>();
        let pairs = paths.iter().map(|(input, output)| (input.as_str(), output.as_str(), false)).collect::<Vec<_>>();
        let compiled = compile_fixture_with_registry(&model, &pairs, &registry)?;
        assert_eq!(tokens(&compiled)?.len(), count);
    }
    let paths = paths.iter().chain(std::iter::once(&"9"))
        .map(|suffix| (format!("Start{suffix}"), format!("Next{suffix}"))).collect::<Vec<_>>();
    let ninth = paths.iter().map(|(input, output)| (input.as_str(), output.as_str(), false)).collect::<Vec<_>>();
    assert!(compile_fixture_with_registry(&model, &ninth, &registry).is_err());
    Ok(())
}
