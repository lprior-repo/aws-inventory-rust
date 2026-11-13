//! arbitrary unicode identifiers obey the byte contract properties.
use super::*;

proptest! {
    #![proptest_config(config())]

    #[test]
    fn arbitrary_unicode_identifiers_obey_the_byte_contract(value in text()) {
        let accepted = !value.is_empty()
            && value.len() <= 4_096
            && !value.chars().any(|character| character == '\0');
        prop_assert_eq!(validate_identifier(&value, "generated").is_ok(), accepted);
    }

    #[test]
    fn all_required_resource_identifiers_are_checked(value in invalid_identifier(), field in 0u8..4) {
        let mut record = resource();
        match field {
            0 => record.id = value,
            1 => record.account_id = value,
            2 => record.region = value,
            _ => record.kind = value,
        }
        prop_assert!(matches!(record.validate(), Err(ValidationError::InvalidField(_))));
    }

    #[test]
    fn all_required_edge_identifiers_are_checked(value in invalid_identifier(), field in 0u8..3) {
        let mut relationship = edge();
        match field {
            0 => relationship.from_id = value,
            1 => relationship.to_id = value,
            _ => relationship.relation = value,
        }
        prop_assert!(matches!(relationship.validate(), Err(ValidationError::InvalidField(_))));
    }

    #[test]
    fn all_scope_fields_are_validated_before_key_creation(value in invalid_identifier(), field in 0u8..4) {
        let mut context = scope();
        match field {
            0 => context.account_id = value,
            1 => context.region = value,
            2 => context.collector = value,
            _ => context.parent_id = Some(value),
        }
        prop_assert!(context.validate().is_err());
        prop_assert!(context.key().is_err());
    }

    #[test]
    fn names_and_tag_values_allow_empty_text_but_reject_nul(value in text()) {
        let valid = !value.chars().any(|character| character == '\0');
        let mut named = resource();
        named.name = Some(value.clone());
        prop_assert_eq!(named.validate().is_ok(), valid);
        let mut tagged = resource();
        tagged.tags.insert("label".into(), value);
        prop_assert_eq!(tagged.validate().is_ok(), valid);
    }

    #[test]
    fn invalid_tag_keys_are_rejected(value in invalid_identifier()) {
        let mut record = resource();
        record.tags.insert(value, String::new());
        prop_assert!(record.validate().is_err());
    }

    #[test]
    fn observation_times_accept_zero_and_all_positive_i64_values(time in any::<i64>()) {
        let mut record = resource();
        record.observed_at = time;
        let mut relationship = edge();
        relationship.observed_at = time;
        prop_assert_eq!(record.validate().is_ok(), !time.is_negative());
        prop_assert_eq!(relationship.validate().is_ok(), !time.is_negative());
    }

    #[test]
    fn tag_cardinality_is_independent_of_contents(count in 0usize..104) {
        let mut record = resource();
        record.tags = (0..count).map(|index| (format!("tag-{index}"), String::new())).collect();
        prop_assert_eq!(record.validate().is_ok(), count <= 100);
    }

    #[test]
    fn bounded_serialization_matches_independent_output_length(value in text(), limit in 0usize..2_048) {
        let object = json!({"escaped": value, "nested": [true, null, 17]});
        let bytes = serde_json::to_vec(&object)?;
        let observed = validate_serialized_size(&object, limit, "budget");
        prop_assert_eq!(observed.is_ok(), bytes.len() <= limit);
        if bytes.len() > limit {
            prop_assert!(matches!(observed, Err(ValidationError::Limit("budget"))));
        }
        prop_assert!(validate_serialized_size(&object, bytes.len(), "exact").is_ok());
        prop_assert!(matches!(validate_serialized_size(&object, bytes.len().saturating_sub(1), "short"), Err(ValidationError::Limit("short"))));
    }

    #[test]
    fn scope_keys_round_trip_and_include_all_identity_fields(account in identifier(), region in identifier(), collector in identifier(), parent in prop::option::of(identifier())) {
        let context = CollectionScope { account_id: account, region, collector, parent_id: parent, source: Source::AwsConfig };
        let key = context.key()?;
        let decoded: CollectionScope = serde_json::from_str(&key)?;
        prop_assert_eq!(&decoded, &context);
        for other_source in [Source::Direct, Source::CloudControl, Source::ResourceExplorer] {
            let mut other = context.clone();
            other.source = other_source;
            prop_assert_ne!(&key, &other.key()?);
        }
    }

    #[test]
    fn page_numbers_use_inclusive_nonzero_bounds(number in prop_oneof![any::<u32>(), Just(0), Just(1), Just(100_000), Just(100_001)]) {
        let mut observation = page();
        observation.page_number = number;
        prop_assert_eq!(observation.validate().is_ok(), (1..=100_000).contains(&number));
    }

    #[test]
    fn continuation_tokens_use_byte_bounds_and_reject_nul(token in prop::option::of(text())) {
        let valid = match &token {
            None => true,
            Some(value) => !value.is_empty() && value.len() <= 8_192 && !value.chars().any(|character| character == '\0'),
        };
        let mut observation = page();
        observation.next_token = token;
        prop_assert_eq!(observation.validate().is_ok(), valid);
    }

    #[test]
    fn page_validation_reaches_invalid_nested_records(value in invalid_identifier(), nested in 0u8..3) {
        let mut observation = page();
        match nested {
            0 => observation.scope.account_id = value,
            1 => { let mut record = resource(); record.id = value; observation.resources = vec![record]; }
            _ => { let mut relationship = edge(); relationship.to_id = value; observation.edges = vec![relationship]; }
        }
        prop_assert!(observation.validate().is_err());
    }

    #[test]
    fn valid_events_preserve_unicode_properties_through_serde(value in text()) {
        let mut observation = page();
        let mut record = resource();
        record.properties = json!({"arbitrary": value});
        observation.resources = vec![record];
        prop_assert!(observation.validate().is_ok());
        let event = CollectionEvent::Page(observation);
        let encoded = serde_json::to_vec(&event)?;
        let decoded: CollectionEvent = serde_json::from_slice(&encoded)?;
        prop_assert_eq!(decoded, event);
    }
}
