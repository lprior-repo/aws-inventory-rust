//! mapper runs once per admitted row and never for an oversized page properties.
use super::*;

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn mapper_runs_once_per_admitted_row_and_never_for_an_oversized_page(count in 0_usize..1_025) {
        let calls = AtomicUsize::new(0);
        let mapped = Page::map((0..count).collect(), None, |index| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(format!("resource-{index}"))
        });
        if count <= MAX_PAGE_ITEMS {
            let mapped = mapped.map_err(|error| TestCaseError::fail(error.to_string()))?;
            prop_assert_eq!(mapped.items.len(), count);
            prop_assert_eq!(calls.load(Ordering::SeqCst), count);
            for (index, item) in mapped.items.into_iter().enumerate() {
                prop_assert_eq!(item.map_err(|error| TestCaseError::fail(error.to_string()))?, format!("resource-{index}"));
            }
        } else {
            prop_assert!(mapped.is_err());
            prop_assert_eq!(calls.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn actual_driver_continues_empty_pages_until_a_terminal_response(empty_pages in 0_u32..32) {
        runtime()?.block_on(async {
            let empty_pages = usize::try_from(empty_pages)?;
            let config = config();
            let executor = ApiExecutor::new(ExecutorConfig::default()).map_err(|error| TestCaseError::fail(error.to_string()))?;
            let client = client(&config, &executor, u32::try_from(empty_pages)?.saturating_add(1));
            let calls = AtomicUsize::new(0);
            let listing = collect(&client, "ListFixture", None, 1, |token| {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                let expected = (call > 0).then(|| format!("cursor-{}", call.saturating_sub(1)));
                assert!(call <= empty_pages, "driver fetched beyond the terminal fixture page");
                assert_eq!(token, expected, "driver changed the expected continuation token");
                let result = if call == empty_pages {
                    Ok(page(vec![Ok("terminal-resource".to_owned())], None))
                } else {
                    Ok(page(vec![], Some(format!("cursor-{call}"))))
                };
                future::ready(result)
            }).await;
            prop_assert_eq!(calls.load(Ordering::SeqCst), empty_pages.saturating_add(1));
            prop_assert_eq!(listing.items, vec!["terminal-resource"]);
            prop_assert_eq!(listing.coverage.pages, u64::try_from(empty_pages)?.saturating_add(1));
            prop_assert_eq!(listing.coverage.items_seen, 1);
            prop_assert!(listing.coverage.complete);
            prop_assert!(listing.coverage.failures.is_empty());
            Ok(())
        })?;
    }

    #[test]
    fn cancellation_retains_a_generated_prefix_and_prevents_the_next_fetch(prefix in 1_u32..32) {
        // Assert the terminal transition before entering the async driver. A
        // broken cancellation transition otherwise spins without another fetch,
        // so a fetch-count assertion alone cannot bound that defective future.
        let mut terminal = Cursor::<String>::new();
        terminal.fail("ListFixture", Some("parent"), &AwsScanError::Api(ApiError::Cancelled));
        prop_assert!(terminal.done, "cancellation must make the owned cursor terminal");
        prop_assert!(terminal.listing.coverage.cancelled);
        runtime()?.block_on(async {
            let prefix_size = usize::try_from(prefix)?;
            let config = config();
            let executor = ApiExecutor::new(ExecutorConfig::default()).map_err(|error| TestCaseError::fail(error.to_string()))?;
            let client = client(&config, &executor, prefix.saturating_add(2));
            let calls = AtomicUsize::new(0);
            let listing = collect(&client, "ListFixture", Some("parent"), prefix_size.saturating_add(2), |token| {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                let expected = (call > 0).then(|| format!("cursor-{}", call.saturating_sub(1)));
                assert!(call < prefix_size, "driver fetched after cancellation");
                assert_eq!(token, expected, "driver changed the expected continuation token");
                if call.saturating_add(1) == prefix_size {
                    executor.cancel();
                }
                future::ready(Ok(page(vec![Ok(format!("resource-{call}"))], Some(format!("cursor-{call}")))))
            }).await;
            prop_assert_eq!(calls.load(Ordering::SeqCst), prefix_size);
            prop_assert_eq!(listing.items, (0..prefix).map(|index| format!("resource-{index}")).collect::<Vec<_>>());
            prop_assert_eq!(listing.coverage.pages, u64::from(prefix));
            prop_assert_eq!(listing.coverage.items_seen, u64::from(prefix));
            prop_assert!(!listing.coverage.complete);
            prop_assert!(listing.coverage.cancelled);
            prop_assert_eq!(listing.coverage.failures.len(), 1);
            let failure = listing.coverage.failures.first()
                .ok_or_else(|| TestCaseError::fail("missing cancellation failure"))?;
            prop_assert_eq!(&failure.category, "cancelled");
            prop_assert_eq!(executor.stats().attempts, 0);
            Ok(())
        })?;
    }

    #[test]
    fn fetch_failure_preserves_the_prefix_and_is_not_counted_as_a_received_page(
        prefix in 0_u32..32,
        denied in any::<bool>(),
    ) {
        runtime()?.block_on(async {
            let prefix_size = usize::try_from(prefix)?;
            let config = config();
            let executor = ApiExecutor::new(ExecutorConfig::default()).map_err(|error| TestCaseError::fail(error.to_string()))?;
            let client = client(&config, &executor, prefix.saturating_add(2));
            let calls = AtomicUsize::new(0);
            let listing = collect(&client, "ListFixture", None, prefix_size.saturating_add(2), |token| {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                let expected = (call > 0).then(|| format!("cursor-{}", call.saturating_sub(1)));
                assert!(call <= prefix_size, "driver fetched after terminal failure");
                assert_eq!(token, expected, "driver changed the expected continuation token");
                let result = if call == prefix_size {
                    Err(AwsScanError::Api(if denied { ApiError::Denied("fixture denial".into()) } else { ApiError::Timeout }))
                } else {
                    Ok(page(vec![Ok(format!("resource-{call}"))], Some(format!("cursor-{call}"))))
                };
                future::ready(result)
            }).await;
            prop_assert_eq!(calls.load(Ordering::SeqCst), prefix_size.saturating_add(1));
            prop_assert_eq!(listing.items, (0..prefix).map(|index| format!("resource-{index}")).collect::<Vec<_>>());
            prop_assert_eq!(listing.coverage.pages, u64::from(prefix));
            prop_assert_eq!(listing.coverage.items_seen, u64::from(prefix));
            prop_assert!(!listing.coverage.complete);
            prop_assert!(!listing.coverage.cancelled);
            prop_assert_eq!(listing.coverage.failures.len(), 1);
            let failure = listing.coverage.failures.first()
                .ok_or_else(|| TestCaseError::fail("missing fetch failure"))?;
            prop_assert_eq!(&failure.category, if denied { "access_denied" } else { "timeout" });
            Ok(())
        })?;
    }
}
