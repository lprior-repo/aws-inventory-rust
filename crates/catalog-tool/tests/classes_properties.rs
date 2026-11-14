use catalog_tool::classes::{Class, DenyList, OperationFacts, classify};
use proptest::{prelude::*, test_runner::RngSeed};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1024, rng_seed: RngSeed::Fixed(0x434c_4153_5345_5301),
        failure_persistence: None, ..ProptestConfig::default()
    })]

    #[test]
    fn allowance_requires_list_or_describe_for_arbitrary_names_and_flags(
        name in prop_oneof![".{0,80}", "List[A-Za-z0-9_]{0,32}", "Describe[A-Za-z0-9_]{0,32}"],
        required in any::<bool>(), readonly in any::<bool>(),
    ) {

        let deny = DenyList::parse("").map_err(|e| TestCaseError::fail(e.to_string()))?;
        let facts = OperationFacts { service: "svc", name: &name,
            has_required_input: required, readonly };
        let class = classify(&facts, &deny);
        if matches!(class, Class::Enumerate | Class::Expand) {
            prop_assert!(name.starts_with("List") || name.starts_with("Describe"));
            prop_assert_eq!(class, if required { Class::Expand } else { Class::Enumerate });
        }
        prop_assert_eq!(class, classify(&OperationFacts { readonly: !readonly, ..facts }, &deny));
    }

    #[test]
    fn matching_denials_never_allow_arbitrary_identifiers_and_flags(
        name in prop_oneof!["[A-Za-z_][A-Za-z0-9_]{0,63}", "List[A-Za-z0-9_]{0,32}",
            "Describe[A-Za-z0-9_]{0,32}"], required in any::<bool>(),
        readonly in any::<bool>(), scoped in any::<bool>(),
    ) {

        let key = if scoped { format!("svc/{name}") } else { name.clone() };
        let text = format!("{key}\tcontent\n");
        let deny = DenyList::parse(&text).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let facts = OperationFacts { service: "svc", name: &name,
            has_required_input: required, readonly };
        prop_assert!(matches!(classify(&facts, &deny), Class::Denied(_)));
    }
}
