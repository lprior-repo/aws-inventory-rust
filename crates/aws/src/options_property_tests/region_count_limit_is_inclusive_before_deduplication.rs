use super::*;

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn region_count_limit_is_inclusive_before_deduplication(region in "[a-z][a-z0-9-]{0,20}") {
        for count in [0, 1, 63, 64, 65, 66] {
            let mut options = options();
            options.regions = vec![region.clone(); count];
            if count <= 64 {
                prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
            } else {
                rejected(&options, "at most 64 regions and 32 service selectors are accepted")?;
            }
        }
    }

    #[test]
    fn selector_count_limit_is_inclusive_before_deduplication(index in 0_usize..SELECTORS.len()) {
        let (selector, collectors) = SELECTORS.get(index).ok_or_else(|| TestCaseError::fail("invalid fixture selector index"))?;
        for count in [1, 31, 32, 33, 34] {
            let mut options = options();
            options.services = vec![(*selector).to_owned(); count];
            if count <= 32 {
                prop_assert_eq!(admitted(&options)?, expected(collectors));
            } else {
                rejected(&options, "at most 64 regions and 32 service selectors are accepted")?;
            }
        }
    }

    #[test]
    fn valid_region_and_selector_caps_can_be_used_together(region in "[a-z][a-z0-9-]{0,20}") {
        let mut options = options();
        options.regions = vec![region; 64];
        options.services = vec!["iam".into(); 32];
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
    }

    #[test]
    fn selector_unions_are_order_and_duplicate_independent(indices in prop::collection::vec(0_usize..SELECTORS.len(), 1..=32)) {
        let mut options = options();
        options.services.clear();
        let mut selected = BTreeSet::new();
        for index in indices {
            let (selector, collectors) = SELECTORS.get(index).ok_or_else(|| TestCaseError::fail("invalid fixture selector index"))?;
            options.services.push((*selector).to_owned());
            selected.extend(expected(collectors));
        }
        prop_assert_eq!(admitted(&options)?, selected.clone());
        options.services.reverse();
        prop_assert_eq!(admitted(&options)?, selected);
        options.services = vec!["all".into(), "ecs".into(), "iam".into()];
        prop_assert_eq!(admitted(&options)?, all_native());
    }

    #[test]
    fn all_never_hides_an_unknown_selector(unknown in "unknown-[a-z0-9]{1,24}") {
        let mut options = options();
        options.services = vec!["all".into(), unknown.clone()];
        rejected(&options, &format!("unsupported service selector or missing supplemental source: {unknown}"))?;
    }

    #[test]
    fn resource_type_selection_requires_explicit_cloud_control(count in 1_usize..8) {
        let mut options = options();
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
        options.selection = inventory_core::Selection::Types(vec![
            inventory_core::ResourceType::new("AWS::EC2::VPC");
            count
        ]);
        rejected(&options, "resource type selection requires Cloud Control")?;
        options.cloud_control = true;
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
        options.selection = inventory_core::Selection::Types(Vec::new());
        prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
    }

    #[test]
    fn optional_text_uses_exact_byte_limits_without_truncating_unicode(unit in prop::sample::select(vec!["a", "é", "中", "🦀"])) {
        for field in TEXT_FIELDS {
            let mut options = options();
            field.assign(&mut options, None);
            prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
            for bytes in [0, 1, 2_047, 2_048, 2_049, 2_050] {
                let value = text_with_bytes(bytes, unit);
                prop_assert_eq!(value.len(), bytes);
                field.assign(&mut options, Some(value));
                if (1..=2_048).contains(&bytes) {
                    prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
                } else {
                    rejected(&options, field.error())?;
                }
            }
        }
    }

    #[test]
    fn nul_in_any_optional_text_is_rejected_without_masking_by_other_limits(prefix in "[a-z0-9]{0,40}", suffix in "[a-z0-9]{0,40}") {
        for field in TEXT_FIELDS {
            let mut options = options();
            field.assign(&mut options, Some(format!("{prefix}x{suffix}")));
            prop_assert_eq!(admitted(&options)?, expected(&["iam"]));
            field.assign(&mut options, Some(format!("{prefix}\0{suffix}")));
            rejected(&options, field.error())?;
        }
    }

    #[test]
    fn region_length_and_ascii_alphabet_are_independent_contracts(valid in "[a-z0-9-]{1,64}", bad in prop::sample::select(vec!["A", "_", " ", "\n", "\0", "é", "１"])) {
        prop_assert!(validate_region(&valid).is_ok());
        prop_assert!(validate_region("").is_err());
        prop_assert!(validate_region(&"a".repeat(64)).is_ok());
        prop_assert!(validate_region(&"a".repeat(65)).is_err());
        prop_assert!(validate_region(&format!("us{bad}east-1")).is_err(), "unsupported region character was admitted");
        let mut options = options();
        options.regions = vec![valid, format!("us{bad}east-1")];
        prop_assert!(matches!(options.validate(), Err(AwsScanError::InvalidOptions(_))), "invalid region was not rejected by scan admission");
    }
}
