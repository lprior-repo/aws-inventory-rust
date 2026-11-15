use catalog_tool::overlay::{OperationName, OperationOverlay, OverlayClass, ServiceOverlay};

fn reviewed(class: OverlayClass, reason: Option<&str>) -> Result<ServiceOverlay, catalog_tool::overlay::NameError> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(class);
    plan.defer_reason = reason.map(str::to_owned);
    ServiceOverlay::new("example", vec![("ListThings", plan)])
}

#[test]
fn deferred_operations_retain_their_review_reason() -> Result<(), Box<dyn std::error::Error>> {
    let overlay = reviewed(OverlayClass::Defer, Some("Identity needs independent review"))?;
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
    for (class, reason) in [
        (OverlayClass::Defer, None),
        (OverlayClass::Defer, Some("")),
        (OverlayClass::Defer, Some("   ")),
        (OverlayClass::Defer, Some("bad\nreason")),
        (OverlayClass::Deny(catalog_tool::classes::DenyReason::Content), Some("Not callable")),
        (OverlayClass::Enumerate, Some("Not callable")),
    ] {
        assert!(reviewed(class, reason).is_err(), "accepted {class:?} with {reason:?}");
    }
}
