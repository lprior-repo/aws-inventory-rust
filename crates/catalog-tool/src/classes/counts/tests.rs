use super::{Class, Counts, Error, OperationFacts};
use std::collections::BTreeMap;

#[test]
fn validate_rejects_operation_total_that_differs_from_class_sum() {
    // Given two operations but only one class, when validating, then totals are rejected.
    let counts = Counts(BTreeMap::from([("operations", 2), ("enumerate", 1)]));
    assert!(matches!(counts.validate(), Err(Error::CountMismatch)));
}

#[test]
fn validate_rejects_denied_total_that_differs_from_reason_sum() {
    // Given a matching operation/class total but no denial reason, when validating,
    // then the separate reason-sum invariant rejects the report.
    let counts = Counts(BTreeMap::from([("operations", 1), ("denied", 1)]));
    assert!(matches!(counts.validate(), Err(Error::CountMismatch)));
}

#[test]
fn record_rejects_nonread_operation_classified_as_enumerate() {
    // Given a non-read name and an inconsistent allow class, when recording,
    // then the builder's non-read/mutating invariant rejects it.
    let facts = OperationFacts {
        service: "test",
        name: "PutThing",
        has_required_input: false,
        readonly: false,
    };
    assert!(matches!(
        Counts::default().record(&facts, Class::Enumerate),
        Err(Error::CountMismatch)
    ));
}
