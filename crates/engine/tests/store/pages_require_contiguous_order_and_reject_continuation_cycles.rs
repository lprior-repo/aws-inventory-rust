use super::*;

#[tokio::test]
pub(super) async fn pages_require_contiguous_order_and_reject_continuation_cycles() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut input = page("ec2", Source::Direct, Vec::new());
    input.page_number = 2;
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.page_number = 1;
    input.next_token = Some("first".to_owned());
    store.apply_page("one", input.clone()).await?;
    input.page_number = 3;
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.page_number = 2;
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.next_token = Some("second".to_owned());
    store.apply_page("one", input.clone()).await?;
    input.page_number = 3;
    input.next_token = Some("first".to_owned());
    assert!(store.apply_page("one", input.clone()).await.is_err());
    input.next_token = None;
    store.apply_page("one", input.clone()).await?;
    let checkpoint = store
        .checkpoint("one", &input.scope)
        .await?
        .ok_or(StoreError::Conflict("missing expected checkpoint"))?;
    assert_eq!(checkpoint.last_page, 3);
    assert!(checkpoint.complete);
    Ok(())
}
