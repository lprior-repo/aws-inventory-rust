use super::*;

pub(super) async fn record_failure(store: &Store, ledger: &mut Ledger) -> TestCaseResult {
    let result = store
        .record_failure(
            SCAN,
            &CollectionFailure {
                scope: scope(),
                category: "synthetic_failure".into(),
                message: "synthetic scoped failure".into(),
            },
        )
        .await;
    prop_assert_eq!(result.is_ok(), ledger.finished.is_none());
    if result.is_ok() {
        ledger.failed = true;
    }
    assert_ledger(store, ledger).await
}

pub(super) fn alter(page: &mut CollectionPage, variant: u8) {
    match variant {
        1 => {
            if let Some(resource) = page.resources.first_mut() {
                resource.properties = json!({"revision":1});
            }
        }
        2 => {
            if let Some(resource) = page.resources.first().cloned() {
                page.resources.push(resource);
            }
        }
        3 => {
            if let Some(edge) = page.edges.first().cloned() {
                page.edges.push(edge);
            }
        }
        4 => page.next_token = Some("next-1".into()),
        5 => page.next_token = None,
        9 => {
            if let Some(resource) = page.resources.first_mut() {
                resource.source = Source::AwsConfig;
            }
        }
        10 => page.page_number = 0,
        _ => {}
    }
}

pub(super) async fn commands(
    store: &Store,
    ledger: &mut Ledger,
    canonical: &[CollectionPage],
    actions: &[(u8, u8)],
) -> TestCaseResult {
    for (selector, variant) in actions.iter().take(16) {
        match variant {
            6 => finish(store, ledger, ScanStatus::Complete).await?,
            7 => finish(store, ledger, ScanStatus::Cancelled).await?,
            8 => record_failure(store, ledger).await?,
            _ => {
                let index = usize::from(*selector)
                    .checked_rem(canonical.len())
                    .ok_or_else(|| failure("empty canonical fixture"))?;
                let mut page = canonical
                    .get(index)
                    .cloned()
                    .ok_or_else(|| failure("missing fixture page"))?;
                alter(&mut page, *variant);
                apply(store, ledger, page).await?;
            }
        }
    }
    Ok(())
}

pub(super) async fn final_commands(
    store: &Store,
    ledger: &mut Ledger,
    canonical: &[CollectionPage],
) -> TestCaseResult {
    for page in canonical.iter().take(5) {
        apply(store, ledger, page.clone()).await?;
    }
    finish(store, ledger, ScanStatus::Complete).await?;

    for page in ledger.committed.clone().into_iter().take(5) {
        store
            .apply_page(SCAN, page.clone())
            .await
            .map_err(failure)?;
        let mut changed = page;
        changed
            .resources
            .first_mut()
            .ok_or_else(|| failure("missing fixture resource"))?
            .observed_at = 999;
        prop_assert!(matches!(
            store.apply_page(SCAN, changed).await,
            Err(StoreError::Conflict(_))
        ));
        assert_ledger(store, ledger).await?;
    }
    Ok(())
}

pub(super) async fn page_case(
    values: &[u16],
    actions: &[(u8, u8)],
    split: usize,
) -> TestCaseResult {
    let folder = tempfile::tempdir().map_err(failure)?;
    let path = folder.path().join("pages.sqlite");
    let canonical = pages(values)?;
    let first = canonical
        .first()
        .cloned()
        .ok_or_else(|| failure("empty page fixture"))?;
    let mut ledger = Ledger {
        committed: vec![first.clone()],
        ..Ledger::default()
    };
    let (before, after) = actions.split_at(split.min(actions.len()));
    let store = Store::open(&path).await.map_err(failure)?;
    let initial = async {
        store.begin_scan(SCAN, 1).await.map_err(failure)?;
        store.apply_page(SCAN, first).await.map_err(failure)?;
        assert_ledger(&store, &ledger).await?;
        commands(&store, &mut ledger, &canonical, before).await
    }
    .await;
    store.close().await;
    initial?;
    let reopened = Store::open(&path).await.map_err(failure)?;
    let result = async {
        assert_ledger(&reopened, &ledger).await?;
        commands(&reopened, &mut ledger, &canonical, after).await?;
        final_commands(&reopened, &mut ledger, &canonical).await
    }
    .await;
    reopened.close().await;
    result
}

pub(super) async fn rollback_case(value: u16, duplicate_edge: bool) -> TestCaseResult {
    let canonical = pages(&[value, value.saturating_add(1)])?;
    let first = canonical
        .first()
        .cloned()
        .ok_or_else(|| failure("first page missing"))?;
    let second = canonical
        .get(1)
        .cloned()
        .ok_or_else(|| failure("second page missing"))?;
    let store = Store::open(":memory:").await.map_err(failure)?;
    let result = async {
        store.begin_scan(SCAN, 1).await.map_err(failure)?;
        let mut ledger = Ledger {
            committed: vec![first.clone()],
            ..Ledger::default()
        };
        store.apply_page(SCAN, first).await.map_err(failure)?;
        let mut broken = second.clone();
        alter(&mut broken, if duplicate_edge { 3 } else { 2 });
        apply(&store, &mut ledger, broken).await?;
        apply(&store, &mut ledger, second).await?;
        finish(&store, &mut ledger, ScanStatus::Complete).await
    }
    .await;
    store.close().await;
    result
}
