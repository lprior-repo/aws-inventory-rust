use super::*;

#[tokio::test]
pub(super) async fn empty_terminal_page_proves_empty_scope_but_unfinished_page_does_not()
-> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("empty", 1).await?;
    store
        .apply_page("empty", page("ec2", Source::Direct, Vec::new()))
        .await?;
    assert_eq!(
        store
            .finish_scan("empty", ScanStatus::Complete, 2)
            .await?
            .completed_scopes,
        1
    );
    assert_eq!(store.scan_status("empty").await?, ScanStatus::Complete);
    store.begin_scan("unfinished", 3).await?;
    let mut unfinished = page("ec2", Source::Direct, Vec::new());
    unfinished.next_token = Some("more".to_owned());
    store.apply_page("unfinished", unfinished.clone()).await?;
    store
        .finish_scan("unfinished", ScanStatus::Complete, 4)
        .await?;
    assert_eq!(store.scan_status("unfinished").await?, ScanStatus::Partial);
    store.begin_scan("no-scopes", 5).await?;
    store
        .finish_scan("no-scopes", ScanStatus::Complete, 6)
        .await?;
    assert_eq!(store.scan_status("no-scopes").await?, ScanStatus::Partial);
    Ok(())
}
