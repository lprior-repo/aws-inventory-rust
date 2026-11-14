use super::{TestResult, paginated, replace};
use catalog_tool::plans::{
    Error, LargestPageSize, Model, Pagination, PaginatorError, Vocabulary, pagination,
};
use serde_json::{Value, json};
#[test]
fn member_range_maximum_overrides_target_range() -> TestResult {
    // Given two ranges, when choosing a maximum, then the member range wins.
    let mut value = paginated();
    replace(
        &mut value,
        "/shapes/n#Input/members/Limit",
        json!({"target":"n#Size", "traits":{"smithy.api#range":{"max":7}}}),
    )?;
    let model = Model::parse(&value, "svc")?;
    let plan = pagination(&model, "n#ListRoots", &Vocabulary::default())?;
    assert_eq!(
        plan.largest_page_size(&model, "n#ListRoots")?,
        Some(LargestPageSize::Known(7))
    );
    Ok(())
}

#[test]
fn absent_maximum_is_unbounded_and_missing_page_size_is_distinct() -> TestResult {
    // Given no maximum, when derived, then it is not fabricated from a minimum or default.
    let mut value = paginated();
    replace(
        &mut value,
        "/shapes/n#Size/traits",
        json!({"smithy.api#range":{"min":1}}),
    )?;
    let model = Model::parse(&value, "svc")?;
    let plan = pagination(&model, "n#ListRoots", &Vocabulary::default())?;
    assert_eq!(
        plan.largest_page_size(&model, "n#ListRoots")?,
        Some(LargestPageSize::Unbounded)
    );
    assert_eq!(
        Pagination::Unpaginated.largest_page_size(&model, "n#ListRoots")?,
        None
    );
    Ok(())
}

#[test]
fn invalid_range_maxima_are_typed_errors() -> TestResult {
    // Given fractional, negative, overflowing and nonnumeric maxima, none becomes a page size.
    [
        json!(1.5),
        json!(-1),
        json!(4294967296_u64),
        json!("7"),
        Value::Null,
    ]
    .into_iter()
    .try_for_each(|max| {
        let mut value = paginated();
        replace(
            &mut value,
            "/shapes/n#Size/traits/smithy.api#range/max",
            max,
        )?;
        let model = Model::parse(&value, "svc")?;
        let plan = pagination(&model, "n#ListRoots", &Vocabulary::default())?;
        assert!(matches!(
            plan.largest_page_size(&model, "n#ListRoots"),
            Err(Error::InvalidPaginator {
                kind: PaginatorError::Range,
                ..
            })
        ));
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn zero_and_u32_maximum_are_accepted_exactly() -> TestResult {
    // Given both u32 boundaries, when derived, then no conversion is lossy.
    [0_u32, u32::MAX].into_iter().try_for_each(|max| {
        let mut value = paginated();
        replace(
            &mut value,
            "/shapes/n#Size/traits/smithy.api#range/max",
            json!(max),
        )?;
        let model = Model::parse(&value, "svc")?;
        let plan = pagination(&model, "n#ListRoots", &Vocabulary::default())?;
        assert_eq!(
            plan.largest_page_size(&model, "n#ListRoots")?,
            Some(LargestPageSize::Known(max))
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn member_range_without_maximum_uses_target_maximum() -> TestResult {
    // Given a member minimum but no maximum, when derived, the target maximum remains usable.
    let mut value = paginated();
    replace(
        &mut value,
        "/shapes/n#Input/members/Limit",
        json!({
        "target":"n#Size","traits":{"smithy.api#range":{"min":1}}}),
    )?;
    let model = Model::parse(&value, "svc")?;
    let plan = pagination(&model, "n#ListRoots", &Vocabulary::default())?;
    assert_eq!(
        plan.largest_page_size(&model, "n#ListRoots")?,
        Some(LargestPageSize::Known(90))
    );
    Ok(())
}
