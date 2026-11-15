use catalog_tool::overlay::{OperationName, ServiceOverlay};

const HEADER: &str =
    "format=1\nreviewed='2026-11-13'\nreviewer='Inventory reviewer'\n[operation.ListThings]\n";

#[test]
fn deferred_operations_retain_their_review_reason() -> Result<(), Box<dyn std::error::Error>> {
    let overlay = ServiceOverlay::parse(
        "example",
        &format!("{HEADER}class='defer'\ndefer_reason='Identity needs independent review'"),
    )?;
    let operation = overlay
        .operations()
        .get(&OperationName::new("ListThings")?)
        .ok_or("missing decision")?;
    assert_eq!(
        operation.defer_reason.as_deref(),
        Some("Identity needs independent review")
    );
    Ok(())
}

#[test]
fn missing_empty_or_misclassified_defer_reasons_are_rejected() {
    for fields in [
        "class='defer'",
        "class='defer'\ndefer_reason=''",
        "class='defer'\ndefer_reason='   '",
        "class='defer'\ndefer_reason=\"bad\\nreason\"",
        "class='deny'\ndeny_reason='content'\ndefer_reason='Not callable'",
        "class='enumerate'\ndefer_reason='Not callable'",
    ] {
        assert!(
            ServiceOverlay::parse("example", &format!("{HEADER}{fields}")).is_err(),
            "accepted {fields}"
        );
    }
}
