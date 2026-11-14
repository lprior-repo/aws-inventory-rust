use catalog_tool::overlay::MemberPath;
use proptest::{prelude::*, test_runner::RngSeed};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1024, rng_seed: RngSeed::Fixed(0x4f56_4552_4c41_5901),
        failure_persistence: None, ..ProptestConfig::default()
    })]

    #[test]
    fn valid_member_paths_round_trip_and_reject_inserted_non_grammar_characters(
        parts in prop::collection::vec("[A-Za-z][A-Za-z0-9_]{0,29}", 1..=8),
        bad in any::<char>().prop_filter("outside path grammar", |c|
            !c.is_ascii_alphanumeric() && !matches!(c, '_' | '.')),
        position in 0usize..256,
    ) {
        // Given valid paths and arbitrary forbidden characters, when inserted, then validation fails.
        let text = parts.join(".");
        let parsed = MemberPath::new(&text).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(parsed.as_str(), text.as_str());
        prop_assert_eq!(parsed.to_string(), text.clone());
        let at = position.min(text.len());
        let prefix: String = text.chars().take(at).collect();
        let suffix: String = text.chars().skip(at).collect();
        let invalid = format!("{prefix}{bad}{suffix}");
        prop_assert_eq!(MemberPath::new(&invalid).err().map(|e| e.value), Some(invalid));
    }
}
