//! Finite pagination models: observed pages survive denial, malformed rows and
//! cancellation, while every nonterminal cursor remains bounded.

use std::sync::atomic::{AtomicUsize, Ordering};

use aws_config::BehaviorVersion;
use aws_types::{SdkConfig, region::Region};
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError};

use super::*;

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x5041_4745_5f41_5753),
        ..Config::default()
    }
}

fn runtime() -> Result<tokio::runtime::Runtime, TestCaseError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .map_err(|error| TestCaseError::fail(error.to_string()))
}

fn client<'a>(
    config: &'a SdkConfig,
    executor: &'a ApiExecutor,
    max_pages: u32,
) -> DiscoveryClient<'a> {
    DiscoveryClient {
        config,
        executor,
        account: "000000000000",
        partition: "aws",
        region: "us-east-1",
        max_pages,
        page_size: 1,
    }
}

fn config() -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .build()
}

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
            let token = (index + 1 < pages.len()).then(|| format!("cursor-{index}"));
            cursor.accept(
                page(rows.iter().map(|item| Ok(format!("resource-{item}"))).collect(), token),
                "ListFixture", Some("parent"), u32::try_from(pages.len())?, raw_count + 1,
            );
            prop_assert_eq!(cursor.done, index + 1 == pages.len());
        }
        let duplicates = raw_count - expected.len();
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
            let terminal = index + 1 == cap;
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
                "ListFixture", None, cap, usize::try_from(cap)? + 1,
            );
            prop_assert_eq!(cursor.done, index + 1 == cap);
        }
        prop_assert_eq!(cursor.listing.items.len(), usize::try_from(cap)?);
        prop_assert_eq!(cursor.listing.coverage.pages, u64::from(cap));
        prop_assert!(!cursor.listing.coverage.complete);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        prop_assert!(cursor.listing.coverage.failures[0].message.contains("page limit"));
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
        prop_assert!(cursor.listing.coverage.failures[0].message.contains("item limit"));
    }

    #[test]
    fn a_page_over_the_raw_item_budget_is_rejected_atomically(
        retained in 1_usize..32,
        remaining in 1_usize..16,
        excess in 1_usize..16,
    ) {
        let mut cursor = Cursor::new();
        let first = (0..retained).map(|index| format!("resource-{index}")).collect::<Vec<_>>();
        cursor.accept(page(first.iter().cloned().map(Ok).collect(), Some("next".into())), "ListFixture", None, 3, retained + remaining);
        prop_assert!(!cursor.done);
        // Even duplicate identities consume the response budget. Deduplicating
        // before admission would permit unbounded raw response processing.
        cursor.accept(page(std::iter::repeat_with(|| Ok("resource-0".to_owned())).take(remaining + excess).collect(), None), "ListFixture", None, 3, retained + remaining);
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items, first);
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::try_from(retained)?);
        prop_assert_eq!(cursor.listing.coverage.pages, 2);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        prop_assert!(!cursor.listing.coverage.complete);
        prop_assert!(cursor.listing.coverage.failures[0].message.contains("would be exceeded"));
    }

    #[test]
    fn every_prior_token_is_checked_for_a_cycle(prefix in 1_usize..48, choose in any::<usize>()) {
        let mut cursor = Cursor::new();
        for index in 0..prefix {
            cursor.accept(page(vec![Ok(format!("resource-{index}"))], Some(format!("cursor-{index}"))), "ListFixture", None, 100, 100);
            prop_assert!(!cursor.done);
        }
        let repeated = choose % prefix;
        cursor.accept(page(vec![Ok("last-observation".to_owned())], Some(format!("cursor-{repeated}"))), "ListFixture", None, 100, 100);
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items.len(), prefix + 1);
        prop_assert_eq!(cursor.seen_tokens.len(), prefix);
        prop_assert_eq!(cursor.token, Some(format!("cursor-{}", prefix - 1)));
        prop_assert!(!cursor.listing.coverage.complete);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), 1);
        prop_assert!(cursor.listing.coverage.failures[0].message.contains("token cycle"));
    }

    #[test]
    fn continuation_limit_counts_utf8_bytes(delta in -4_i32..=4) {
        let byte_length = usize::try_from(i64::try_from(MAX_TOKEN_BYTES)? + i64::from(delta))?;
        let token = format!("{}{}", "🦀".repeat(byte_length / 4), "x".repeat(byte_length % 4));
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
        let invalid = if oversized { "é".repeat(MAX_METADATA_BYTES / 2 + 1) } else { String::new() };
        let rows = expected.iter().cloned().map(Ok)
            .chain(std::iter::repeat_with(|| Ok(invalid.clone())).take(bad_count))
            .chain(std::iter::once(Err(data_error("invalid mapped fixture")))).collect::<Vec<_>>();
        let length = rows.len();
        let mut cursor = Cursor::new();
        cursor.accept(page(rows, None), "ListFixture", Some("parent"), 1, length);
        prop_assert!(cursor.done);
        prop_assert_eq!(cursor.listing.items, expected);
        prop_assert_eq!(cursor.listing.coverage.items_seen, u64::try_from(length)?);
        prop_assert_eq!(cursor.listing.coverage.failures.len(), bad_count + 1);
        prop_assert!(!cursor.listing.coverage.complete);
    }

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
            let client = client(&config, &executor, u32::try_from(empty_pages)? + 1);
            let calls = AtomicUsize::new(0);
            let listing = collect(&client, "ListFixture", None, 1, |token| {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                let expected = (call > 0).then(|| format!("cursor-{}", call - 1));
                assert!(call <= empty_pages, "driver fetched beyond the terminal fixture page");
                assert_eq!(token, expected, "driver changed the expected continuation token");
                let result = if call == empty_pages {
                    Ok(page(vec![Ok("terminal-resource".to_owned())], None))
                } else {
                    Ok(page(vec![], Some(format!("cursor-{call}"))))
                };
                future::ready(result)
            }).await;
            prop_assert_eq!(calls.load(Ordering::SeqCst), empty_pages + 1);
            prop_assert_eq!(listing.items, vec!["terminal-resource"]);
            prop_assert_eq!(listing.coverage.pages, u64::try_from(empty_pages)? + 1);
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
            let client = client(&config, &executor, prefix + 2);
            let calls = AtomicUsize::new(0);
            let listing = collect(&client, "ListFixture", Some("parent"), prefix_size + 2, |token| {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                let expected = (call > 0).then(|| format!("cursor-{}", call - 1));
                assert!(call < prefix_size, "driver fetched after cancellation");
                assert_eq!(token, expected, "driver changed the expected continuation token");
                if call + 1 == prefix_size {
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
            prop_assert_eq!(&listing.coverage.failures[0].category, "cancelled");
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
            let client = client(&config, &executor, prefix + 2);
            let calls = AtomicUsize::new(0);
            let listing = collect(&client, "ListFixture", None, prefix_size + 2, |token| {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                let expected = (call > 0).then(|| format!("cursor-{}", call - 1));
                assert!(call <= prefix_size, "driver fetched after terminal failure");
                assert_eq!(token, expected, "driver changed the expected continuation token");
                let result = if call == prefix_size {
                    Err(AwsScanError::Api(if denied { ApiError::Denied("fixture denial".into()) } else { ApiError::Timeout }))
                } else {
                    Ok(page(vec![Ok(format!("resource-{call}"))], Some(format!("cursor-{call}"))))
                };
                future::ready(result)
            }).await;
            prop_assert_eq!(calls.load(Ordering::SeqCst), prefix_size + 1);
            prop_assert_eq!(listing.items, (0..prefix).map(|index| format!("resource-{index}")).collect::<Vec<_>>());
            prop_assert_eq!(listing.coverage.pages, u64::from(prefix));
            prop_assert_eq!(listing.coverage.items_seen, u64::from(prefix));
            prop_assert!(!listing.coverage.complete);
            prop_assert!(!listing.coverage.cancelled);
            prop_assert_eq!(listing.coverage.failures.len(), 1);
            prop_assert_eq!(&listing.coverage.failures[0].category, if denied { "access_denied" } else { "timeout" });
            Ok(())
        })?;
    }
}

#[test]
fn empty_cursor_is_failure_even_when_all_rows_are_valid() {
    let mut cursor = Cursor::new();
    cursor.accept(
        page(vec![Ok("resource".to_owned())], Some(String::new())),
        "ListFixture",
        None,
        2,
        2,
    );
    assert!(cursor.done);
    assert!(!cursor.listing.coverage.complete);
    assert_eq!(cursor.listing.items, vec!["resource"]);
    assert!(cursor.seen_tokens.is_empty());
}

#[test]
fn exact_identity_byte_limit_accepts_unicode_without_truncation() {
    let identity = "é".repeat(MAX_METADATA_BYTES / 2);
    let mut cursor = Cursor::new();
    cursor.accept(
        page(vec![Ok(identity.clone())], None),
        "ListFixture",
        None,
        1,
        1,
    );
    assert!(cursor.listing.coverage.complete);
    assert_eq!(cursor.listing.items, vec![identity]);
}
