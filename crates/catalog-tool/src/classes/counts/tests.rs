use super::{Class, Counts, Error, OperationFacts};
use std::collections::BTreeMap;

#[test]
fn validate_rejects_operation_total_that_differs_from_class_sum() {
    let counts = Counts(BTreeMap::from([("operations", 2), ("enumerate", 1)]));
    assert!(matches!(counts.validate(), Err(Error::CountMismatch)));
}

#[test]
fn validate_rejects_denied_total_that_differs_from_reason_sum() {
    let counts = Counts(BTreeMap::from([("operations", 1), ("denied", 1)]));
    assert!(matches!(counts.validate(), Err(Error::CountMismatch)));
}

#[test]
fn record_rejects_nonread_operation_classified_as_enumerate() {
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
