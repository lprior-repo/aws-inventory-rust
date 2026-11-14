use super::*;

#[derive(Default)]
pub(super) struct Ledger {
    pub(super) committed: Vec<CollectionPage>,
    pub(super) failed: bool,
    pub(super) finished: Option<ScanStatus>,
}

impl Ledger {
    pub(super) fn accepts(&self, page: &CollectionPage) -> bool {
        if page.page_number == 0
            || page
                .resources
                .iter()
                .any(|row| row.source != Source::Direct)
        {
            return false;
        }
        if let Some(previous) = self
            .committed
            .iter()
            .find(|old| old.page_number == page.page_number)
        {
            return previous == page;
        }
        let resource_ids: BTreeSet<_> = page.resources.iter().map(|row| &row.id).collect();
        let edge_ids: BTreeSet<_> = page
            .edges
            .iter()
            .map(|row| (&row.from_id, &row.to_id, &row.relation))
            .collect();
        let expected = self
            .committed
            .last()
            .map_or(1, |last| last.page_number.saturating_add(1));
        let scope_done = self
            .committed
            .last()
            .is_some_and(|last| last.next_token.is_none());
        let repeated_cursor = page.next_token.as_ref().is_some_and(|token| {
            self.committed
                .iter()
                .any(|old| old.next_token.as_ref() == Some(token))
        });
        self.finished.is_none()
            && !scope_done
            && !repeated_cursor
            && page.page_number == expected
            && resource_ids.len() == page.resources.len()
            && edge_ids.len() == page.edges.len()
    }

    pub(super) fn effective_status(&self, requested: ScanStatus) -> ScanStatus {
        if requested == ScanStatus::Complete
            && (self.failed
                || !self
                    .committed
                    .last()
                    .is_some_and(|last| last.next_token.is_none()))
        {
            ScanStatus::Partial
        } else {
            requested
        }
    }
}

pub(super) async fn assert_ledger(store: &Store, ledger: &Ledger) -> TestCaseResult {
    let mut expected_resources: Vec<_> = ledger
        .committed
        .iter()
        .flat_map(|page| page.resources.iter().cloned())
        .collect();
    expected_resources.sort_by(|left, right| left.id.cmp(&right.id));
    let mut expected_edges: Vec<_> = ledger
        .committed
        .iter()
        .flat_map(|page| page.edges.iter().cloned())
        .collect();
    expected_edges.sort_by(|left, right| {
        (&left.from_id, &left.to_id, &left.relation).cmp(&(
            &right.from_id,
            &right.to_id,
            &right.relation,
        ))
    });
    prop_assert_eq!(
        store.resources(SCAN).await.map_err(failure)?,
        expected_resources.clone()
    );
    prop_assert_eq!(
        store.edges(SCAN).await.map_err(failure)?,
        expected_edges.clone()
    );
    let last = ledger
        .committed
        .last()
        .ok_or_else(|| failure("fixture ledger must start nonempty"))?;
    let expected_checkpoint = Checkpoint {
        last_page: last.page_number,
        next_token: last.next_token.clone(),
        complete: last.next_token.is_none(),
        failed: ledger.failed,
    };
    prop_assert_eq!(
        store.checkpoint(SCAN, &scope()).await.map_err(failure)?,
        Some(expected_checkpoint)
    );
    let summary = store.summary(SCAN).await.map_err(failure)?;
    prop_assert_eq!(
        summary.resources,
        u64::try_from(expected_resources.len()).map_err(failure)?
    );
    prop_assert_eq!(
        summary.edges,
        u64::try_from(expected_edges.len()).map_err(failure)?
    );
    prop_assert_eq!(
        summary.completed_scopes,
        u64::from(last.next_token.is_none() && !ledger.failed)
    );
    prop_assert_eq!(summary.failed_scopes, u64::from(ledger.failed));
    prop_assert_eq!(
        store.scan_status(SCAN).await.map_err(failure)?,
        ledger.finished.unwrap_or(ScanStatus::Running)
    );
    prop_assert_eq!(
        store
            .query("SELECT count(*) AS pages FROM collection_pages", 1)
            .await
            .map_err(failure)?,
        json!([{"pages":ledger.committed.len()}])
    );
    prop_assert_eq!(
        store
            .query("SELECT count(*) AS failures FROM collection_failures", 1)
            .await
            .map_err(failure)?,
        json!([{"failures":usize::from(ledger.failed)}])
    );
    Ok(())
}

pub(super) async fn apply(
    store: &Store,
    ledger: &mut Ledger,
    page: CollectionPage,
) -> TestCaseResult {
    let accepted = ledger.accepts(&page);
    let result = store.apply_page(SCAN, page.clone()).await;
    prop_assert_eq!(
        result.is_ok(),
        accepted,
        "page {:?}; result {:?}",
        page,
        result
    );
    if accepted
        && !ledger
            .committed
            .iter()
            .any(|old| old.page_number == page.page_number)
    {
        ledger.committed.push(page);
    }
    assert_ledger(store, ledger).await
}

pub(super) async fn finish(
    store: &Store,
    ledger: &mut Ledger,
    requested: ScanStatus,
) -> TestCaseResult {
    let effective = ledger.effective_status(requested);
    let accepted = ledger.finished.is_none_or(|status| status == effective);
    let result = store.finish_scan(SCAN, requested, 1_000).await;
    prop_assert_eq!(result.is_ok(), accepted);
    if accepted {
        ledger.finished = Some(effective);
    }
    assert_ledger(store, ledger).await
}
