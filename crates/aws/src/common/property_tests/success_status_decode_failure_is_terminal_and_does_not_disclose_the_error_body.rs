use super::*;

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn success_status_decode_failure_is_terminal_and_does_not_disclose_the_error_body(
        status in 200_u16..300,
        suffix in "[a-z0-9]{1,64}",
    ) {
        let secret = format!("response-body-canary-{suffix}");
        let error = classify_sdk_error(modeled(status, None, None, &secret)?);
        match error {
            ApiError::Permanent(message) => {
                prop_assert!(message.contains("response decoding"));
                prop_assert!(!message.contains(&secret));
            }
            other => prop_assert!(false, "success response was misclassified: {other:?}"),
        }
    }

    #[test]
    fn service_diagnostics_are_bounded_by_characters_without_splitting_utf8(message in prop::collection::vec(any::<char>(), 1_010..1_100)) {
        let text: String = message.into_iter().collect();
        let error = classify_sdk_error(modeled(400, Some("FixtureError"), None, &text)?);
        match error {
            ApiError::Permanent(detail) => {
                prop_assert_eq!(detail.chars().count(), 1_024);
                prop_assert!(detail.starts_with("FixtureError: "));
                let expected: String = format!("FixtureError: {text}").chars().take(1_024).collect();
                prop_assert_eq!(detail, expected);
            }
            other => prop_assert!(false, "unexpected classification: {other:?}"),
        }
    }

    #[test]
    fn child_budget_rejects_both_scope_and_single_page_overflow(count in 0_usize..25_000, remaining in 0_usize..150_000) {
        let allowed = count <= remaining && count <= inventory_core::MAX_PAGE_RESOURCES;
        prop_assert_eq!(require_budget(count, remaining).is_ok(), allowed);
        if count > remaining {
            prop_assert!(matches!(require_budget(count, remaining), Err(AwsScanError::ChildLimit)));
        }
    }

    #[test]
    fn terminal_empty_page_completes_at_the_exact_page_limit(tokens in prop::collection::btree_set("[a-z0-9]{1,16}", 0..12)) {
        let mut cursor = Cursor::default();
        let cap = u32::try_from(tokens.len().checked_add(1).ok_or_else(|| TestCaseError::fail("token count overflow"))?)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        for token in &tokens {
            cursor.advance(&PageBody { next_token: Some(token.clone()), ..PageBody::default() }, cap);
            prop_assert!(!cursor.done);
            prop_assert!(cursor.pending_failure.is_none());
        }
        cursor.advance(&PageBody::default(), cap);
        prop_assert!(cursor.done);
        prop_assert!(cursor.pending_failure.is_none());
        prop_assert_eq!(cursor.page_number, cap);
        prop_assert_eq!(cursor.token, None);
        prop_assert_eq!(cursor.seen.len(), tokens.len());
        prop_assert_eq!(cursor.token_bytes, tokens.iter().map(String::len).sum::<usize>());
    }

    #[test]
    fn any_previously_seen_cursor_is_a_cycle(tokens in prop::collection::btree_set("[a-z0-9]{1,16}", 1..12), repeat in any::<usize>()) {
        let mut cursor = Cursor::default();
        for token in &tokens {
            cursor.advance(&PageBody { next_token: Some(token.clone()), ..PageBody::default() }, 100);
        }
        let replay_index = repeat
            .checked_rem(tokens.len())
            .ok_or_else(|| TestCaseError::fail("missing replay token"))?;
        let replay = tokens.iter().nth(replay_index).ok_or_else(|| TestCaseError::fail("missing replay token"))?;
        cursor.advance(&PageBody { next_token: Some(replay.clone()), ..PageBody::default() }, 100);
        prop_assert!(matches!(cursor.pending_failure, Some(("pagination_cycle", _))));
        prop_assert_eq!(cursor.seen.len(), tokens.len());
        prop_assert_eq!(cursor.token.as_deref(), Some(replay.as_str()));
    }

    #[test]
    fn page_cap_preserves_a_nonterminal_checkpoint(cap in 1_u32..32) {
        let mut cursor = Cursor::default();
        for page in 1..=cap {
            cursor.advance(&PageBody { next_token: Some(format!("page-{page}")), ..PageBody::default() }, cap);
        }
        prop_assert_eq!(cursor.page_number, cap);
        prop_assert_eq!(cursor.token, Some(format!("page-{cap}")));
        prop_assert!(!cursor.done);
        prop_assert!(matches!(cursor.pending_failure, Some(("page_limit", _))));
        prop_assert_eq!(cursor.seen.len(), usize::try_from(cap.saturating_sub(1)).map_err(|error| TestCaseError::fail(error.to_string()))?);
    }

    #[test]
    fn continuation_limit_counts_utf8_bytes(characters in 4_097_usize..4_130) {
        let token = "λ".repeat(characters);
        prop_assert!(token.chars().count() < MAX_TOKEN_BYTES);
        let mut cursor = Cursor::default();
        cursor.advance(&PageBody { next_token: Some(token), ..PageBody::default() }, 10);
        prop_assert!(matches!(cursor.pending_failure, Some(("invalid_pagination", _))));
        prop_assert!(cursor.seen.is_empty());
        prop_assert_eq!(cursor.token_bytes, 0);
    }

    #[test]
    fn fetch_failure_preserves_all_preceding_nonterminal_pages(preceding in 0_u8..8, denied in any::<bool>()) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(16);
        let ctx = Context {
            config: SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default()).map_err(|error| TestCaseError::fail(error.to_string()))?),
            sender,
            account: "123456789012".into(), partition: "aws".into(), region: "us-east-1".into(),
            max_pages: 20, page_size: 100,
        };
        let calls = AtomicUsize::new(0);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_time().start_paused(true).build()
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        runtime.block_on(paginate(&ctx, ctx.scope("property_fixture", None, Source::Direct), |_, remaining| {
            let index = calls.fetch_add(1, Ordering::SeqCst);
            assert!(index <= usize::from(preceding), "driver fetched after terminal failure");
            assert_eq!(remaining, MAX_CHILDREN);
            async move {
                if index == usize::from(preceding) {
                    return Err(AwsScanError::Api(if denied { ApiError::Denied("fixture".into()) } else { ApiError::Timeout }));
                }
                Ok(PageBody { next_token: Some(format!("page-{index}")), ..PageBody::default() })
            }
        })).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let events: Vec<_> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
        prop_assert_eq!(calls.load(Ordering::SeqCst), usize::from(preceding) + 1);
        prop_assert_eq!(events.len(), usize::from(preceding) + 1);
        prop_assert_eq!(events.iter().filter(|event| matches!(event, CollectionEvent::Page(page) if page.next_token.is_some())).count(), usize::from(preceding));
        let last = events.last().ok_or_else(|| TestCaseError::fail("missing failure event"))?;
        let category = if denied { "access_denied" } else { "timeout" };
        prop_assert!(matches!(last, CollectionEvent::Failure(failure) if failure.category == category));
    }
}
