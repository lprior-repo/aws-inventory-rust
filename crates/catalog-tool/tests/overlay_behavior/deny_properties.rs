use super::*;
use proptest::{prelude::*, test_runner::RngSeed};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        rng_seed: RngSeed::Fixed(0x4f56_4552_4c41_5903),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn classifier_denials_never_become_callable_under_valid_overrides(
        class in prop_oneof![Just(OverlayClass::Enumerate), Just(OverlayClass::Expand)],
        items in prop_oneof![Just("Items"), Just("$")],
    ) {
        let root = fixture(BASIC, "ListThings\tcontent\n")
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let mut plan = complete()
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        plan.class = Some(class);
        plan.items = Some(ItemSelector::parse(items)
            .map_err(|error| TestCaseError::fail(error.to_string()))?);
        match checked(&root, vec![("ListThings", plan)]).map_err(|error| *error) {
            Ok(checked) => {
                prop_assert!(checked.services().iter()
                    .flat_map(|service| service.callable())
                    .all(|callable| callable.operation() != "ListThings"));
            }
            Err(CheckError::Violations(_)) => (),
            Err(error) => return Err(TestCaseError::fail(error.to_string())),
        }
    }
}
