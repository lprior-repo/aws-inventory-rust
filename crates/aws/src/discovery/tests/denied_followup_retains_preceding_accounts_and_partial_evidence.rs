use super::*;

#[tokio::test]
async fn denied_followup_retains_preceding_accounts_and_partial_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let listing = pages::collect(
        &client,
        "organizations:ListAccounts",
        None,
        10,
        |token| async move {
            if token.is_none() {
                Ok(pages::page(
                    vec![Ok("first".to_owned())],
                    Some("next".into()),
                ))
            } else {
                Err(AwsScanError::Api(ApiError::Denied("fixture denial".into())))
            }
        },
    )
    .await;
    assert_eq!(listing.items, vec!["first"]);
    assert!(!listing.coverage.complete);
    assert_eq!(
        listing
            .coverage
            .failures
            .first()
            .map(|failure| failure.operation.as_str()),
        Some("organizations:ListAccounts")
    );
    Ok(())
}

#[tokio::test]
async fn malformed_and_duplicate_items_do_not_discard_valid_siblings()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let listing = pages::collect(&client, "fixture", None, 10, |_| async {
        Ok(pages::page(
            vec![
                Ok("first".to_owned()),
                Err(data_error("malformed")),
                Ok("first".to_owned()),
                Ok("last".to_owned()),
            ],
            None,
        ))
    })
    .await;
    assert_eq!(listing.items, vec!["first", "last"]);
    assert_eq!(listing.coverage.items_seen, 4);
    assert_eq!(listing.coverage.failures.len(), 2);
    assert!(!listing.coverage.complete);
    Ok(())
}

#[tokio::test]
async fn continuation_cycles_and_page_limits_are_explicit_failures()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let mut client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        async { Ok(pages::page::<String>(vec![], Some("cycle".into()))) }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(!listing.coverage.complete);
    assert!(
        listing
            .coverage
            .failures
            .iter()
            .any(|failure| failure.message.contains("cycle"))
    );
    client.max_pages = 1;
    let limited = pages::collect(&client, "fixture", None, 10, |_| async {
        Ok(pages::page(
            vec![Ok("retained".to_owned())],
            Some("more".into()),
        ))
    })
    .await;
    assert_eq!(limited.items, vec!["retained"]);
    assert!(!limited.coverage.complete);
    assert!(
        limited
            .coverage
            .failures
            .iter()
            .any(|failure| failure.message.contains("page limit"))
    );
    Ok(())
}

#[tokio::test]
async fn precancelled_pagination_makes_no_fetch_call() -> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    executor.cancel();
    let client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        async { Ok(pages::page::<String>(vec![], None)) }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(listing.coverage.cancelled);
    assert!(!listing.coverage.complete);
    assert!(listing.items.is_empty());
    Ok(())
}

#[tokio::test]
async fn cancellation_between_pages_preserves_completed_results()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        executor.cancel();
        async {
            Ok(pages::page(
                vec![Ok("retained".to_owned())],
                Some("more".into()),
            ))
        }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(listing.items, vec!["retained"]);
    assert!(listing.coverage.cancelled);
    assert!(!listing.coverage.complete);
    Ok(())
}

#[tokio::test]
async fn item_limit_and_invalid_token_never_claim_complete()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let limited = pages::collect(&client, "fixture", None, 1, |_| async {
        Ok(pages::page(
            vec![Ok("one".to_owned()), Ok("two".to_owned())],
            None,
        ))
    })
    .await;
    assert!(limited.items.is_empty());
    assert!(!limited.coverage.complete);
    let invalid_token = pages::collect(&client, "fixture", None, 10, |_| async {
        Ok(pages::page::<String>(vec![], Some(String::new())))
    })
    .await;
    assert!(!invalid_token.coverage.complete);
    Ok(())
}

#[test]
fn oversized_sdk_page_is_rejected_before_mapping() {
    let mapped = AtomicUsize::new(0);
    let result = Page::map(vec!["fixture"; 1_001], None, |value| {
        mapped.fetch_add(1, Ordering::SeqCst);
        Ok(value.to_owned())
    });
    assert!(result.is_err());
    assert_eq!(mapped.load(Ordering::SeqCst), 0);
}

#[test]
fn failure_diagnostic_caps_preserve_suppressed_count_and_partial_status() {
    let mut coverage = DiscoveryCoverage::success();
    for _ in 0..MAX_FAILURES.saturating_add(7) {
        coverage.fail("fixture", None, &data_error("failure"));
    }
    assert_eq!(coverage.failures.len(), MAX_FAILURES);
    assert_eq!(coverage.suppressed_failures, 7);
    assert!(!coverage.complete);
    let mut combined = DiscoveryCoverage::success();
    combined.merge(coverage.clone());
    combined.merge(coverage);
    assert_eq!(combined.failures.len(), MAX_FAILURES);
    assert_eq!(
        combined.suppressed_failures,
        u64::try_from(MAX_FAILURES).unwrap_or(0).saturating_add(14)
    );
    assert!(!combined.complete);
}
