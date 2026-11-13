//! discovery keeps the first observation of every identity properties.
use super::*;

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn discovery_keeps_the_first_observation_of_every_identity(
        pages in prop::collection::vec(prop::collection::vec(0_u16..64, 0..20), 1..16),
    ) {
        let raw_count = pages.iter().map(Vec::len).sum::<usize>();
        let mut expected = Vec::new();
        let mut seen = BTreeSet::new();
        for item in pages.iter().flatten() {
            if seen.insert(*item) {
                expected.push(format!("resource-{item}"));
            }
        }
        let mut cursor = Cursor::new();
        for (index, rows) in pages.iter().enumerate() {
            let token = (index.saturating_add(1) < pages.len()).then(|| format!("cursor-{index}"));
            cursor.accept(
                page(rows.iter().map(|item| Ok(format!("resource-{item}"))).collect(), token),
                "ListFixture", Some("parent"), u32::try_from(pages.len())?, raw_count.saturating_add(1),
            );
            prop_assert_eq!(cursor.done, index.saturating_add(1) == pages.len());
        }
        let duplicates = raw_count.saturating_sub(expected.len());
        prop_assert_eq!(&cursor.listing.items, &expected);
        prop_assert_eq!(cursor.listing.coverage.pages, u64::try_from(pages.len())?);
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::try_from(raw_count)?);
        prop_assert_eq!(cursor.listing.coverage.complete, duplicates == 0);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), duplicates);
        prop_assert!(!cursor.listing.coverage.cancelled);
        for failure in &cursor.listing.coverage.failures {
            prop_assert_eq!(&failure.operation, "ListFixture");
            prop_assert_eq!(failure.parent_id.as_deref(), Some("parent"));
            prop_assert_eq!(&failure.category, "invalid_response");
            prop_assert!(failure.message.contains("repeated an item identity"));
        }
    }

    #[test]
    fn terminal_response_at_the_exact_page_and_item_caps_is_complete(cap in 1_u32..64) {
        let mut cursor = Cursor::new();
        for index in 0..cap {
            let terminal = index.saturating_add(1) == cap;
            cursor.accept(
                page(vec![Ok(format!("resource-{index}"))], (!terminal).then(|| format!("cursor-{index}"))),
                "ListFixture", None, cap, usize::try_from(cap)?,
            );
            prop_assert_eq!(cursor.done, terminal);
        }
        prop_assert_eq!(cursor.listing.items.len(), usize::try_from(cap)?);
        prop_assert_eq!(cursor.listing.coverage.pages, u64::from(cap));
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::from(cap));
        prop_assert!(cursor.listing.coverage.complete);
        prop_assert!(cursor.listing.coverage.failures.is_empty());
    }

    #[test]
    fn continuation_at_the_page_cap_reports_incomplete_work(cap in 1_u32..64) {
        let mut cursor = Cursor::new();
        for index in 0..cap {
            cursor.accept(
                page(vec![Ok(format!("resource-{index}"))], Some(format!("cursor-{index}"))),
                "ListFixture", None, cap, usize::try_from(cap)?.saturating_add(1),
            );
            prop_assert_eq!(cursor.done, index.saturating_add(1) == cap);
        }
        prop_assert_eq!(cursor.listing.items.len(), usize::try_from(cap)?);
        prop_assert_eq!(cursor.listing.coverage.pages, u64::from(cap));
        prop_assert!(!cursor.listing.coverage.complete);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        let failure = cursor.listing.coverage.failures.first()
            .ok_or_else(|| TestCaseError::fail("missing page-limit failure"))?;
        prop_assert!(failure.message.contains("page limit"));
    }

    #[test]
    fn continuation_at_the_item_cap_reports_incomplete_work(cap in 1_usize..64) {
        let mut cursor = Cursor::new();
        cursor.accept(
            page((0..cap).map(|index| Ok(format!("resource-{index}"))).collect(), Some("next".into())),
            "ListFixture", None, 2, cap,
        );
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items.len(), cap);
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::try_from(cap)?);
        prop_assert!(!cursor.listing.coverage.complete);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        let failure = cursor.listing.coverage.failures.first()
            .ok_or_else(|| TestCaseError::fail("missing item-limit failure"))?;
        prop_assert!(failure.message.contains("item limit"));
    }

    #[test]
    fn a_page_over_the_raw_item_budget_is_rejected_atomically(
        retained in 1_usize..32,
        remaining in 1_usize..16,
        excess in 1_usize..16,
    ) {
        let mut cursor = Cursor::new();
        let first = (0..retained).map(|index| format!("resource-{index}")).collect::<Vec<_>>();
        cursor.accept(page(first.iter().cloned().map(Ok).collect(), Some("next".into())), "ListFixture", None, 3, retained.saturating_add(remaining));
        prop_assert!(!cursor.done);
        // Even duplicate identities consume the response budget. Deduplicating
        // before admission would permit unbounded raw response processing.
        cursor.accept(page(std::iter::repeat_with(|| Ok("resource-0".to_owned())).take(remaining.saturating_add(excess)).collect(), None), "ListFixture", None, 3, retained.saturating_add(remaining));
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items, first);
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::try_from(retained)?);
        prop_assert_eq!(cursor.listing.coverage.pages, 2);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        prop_assert!(!cursor.listing.coverage.complete);
        let failure = cursor.listing.coverage.failures.first()
            .ok_or_else(|| TestCaseError::fail("missing budget failure"))?;
        prop_assert!(failure.message.contains("would be exceeded"));
    }

    #[test]
    fn every_prior_token_is_checked_for_a_cycle(prefix in 1_usize..48, choose in any::<usize>()) {
        let mut cursor = Cursor::new();
        for index in 0..prefix {
            cursor.accept(page(vec![Ok(format!("resource-{index}"))], Some(format!("cursor-{index}"))), "ListFixture", None, 100, 100);
            prop_assert!(!cursor.done);
        }
        let repeated = choose
            .checked_rem(prefix)
            .ok_or_else(|| TestCaseError::fail("missing cycle token"))?;
        cursor.accept(page(vec![Ok("last-observation".to_owned())], Some(format!("cursor-{repeated}"))), "ListFixture", None, 100, 100);
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items.len(), prefix.saturating_add(1));
        prop_assert_eq!(cursor.seen_tokens.len(), prefix);
        prop_assert_eq!(cursor.token, Some(format!("cursor-{}", prefix.saturating_sub(1))));
        prop_assert!(!cursor.listing.coverage.complete);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        let failure = cursor.listing.coverage.failures.first()
            .ok_or_else(|| TestCaseError::fail("missing token-cycle failure"))?;
        prop_assert!(failure.message.contains("token cycle"));
    }

    #[test]
    fn continuation_limit_counts_utf8_bytes(delta in -4_i32..=4) {
        let byte_length = usize::try_from(
            i64::try_from(MAX_TOKEN_BYTES)?
                .checked_add(i64::from(delta))
                .ok_or_else(|| TestCaseError::fail("token byte length overflow"))?,
        )?;
        let token = format!(
            "{}{}",
            "🦀".repeat(byte_length.div_euclid(4)),
            "x".repeat(byte_length.rem_euclid(4))
        );
        prop_assert_eq!(token.len(), byte_length);
        let mut cursor = Cursor::<String>::new();
        cursor.accept(page(vec![], Some(token.clone())), "ListFixture", None, 2, 1);
        if byte_length <= MAX_TOKEN_BYTES {
            prop_assert!(!cursor.done);
            prop_assert!(cursor.listing.coverage.complete);
            prop_assert_eq!(cursor.token, Some(token));
        } else {
            prop_assert!(cursor.done);
            prop_assert!(!cursor.listing.coverage.complete);
            prop_assert!(cursor.token.is_none());
            prop_assert!(cursor.seen_tokens.is_empty());
        }
    }

    #[test]
    fn malformed_identities_do_not_discard_valid_siblings(
        valid in prop::collection::btree_set(0_u16..1_000, 1..24),
        bad_count in 1_usize..8,
        oversized in any::<bool>(),
    ) {
        let expected = valid.iter().map(|id| format!("resource-{id}")).collect::<Vec<_>>();
        let invalid_length = MAX_METADATA_BYTES
            .checked_div(2)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| TestCaseError::fail("invalid fixture length overflow"))?;
        let invalid = if oversized { "é".repeat(invalid_length) } else { String::new() };
        let rows = expected.iter().cloned().map(Ok)
            .chain(std::iter::repeat_with(|| Ok(invalid.clone())).take(bad_count))
            .chain(std::iter::once(Err(data_error("invalid mapped fixture")))).collect::<Vec<_>>();
        let length = rows.len();
        let mut cursor = Cursor::new();
        cursor.accept(page(rows, None), "ListFixture", Some("parent"), 1, length);
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items, expected);
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::try_from(length)?);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), bad_count.saturating_add(1));
        prop_assert!(!cursor.listing.coverage.complete);
    }

}
