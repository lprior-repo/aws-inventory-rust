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
        class in prop_oneof![Just("enumerate"), Just("expand")],
        items in prop_oneof![Just("Items"), Just("$")],
    ) {
        let root = fixture(BASIC, "ListThings\tcontent\n")
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let text = format!(
            "{HEADER}[operation.ListThings]\nclass='{class}'\nitems='{items}'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n"
        );
        overlay(root.path(), &text)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        match overlay::check(root.path(), &source(root.path())) {
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
