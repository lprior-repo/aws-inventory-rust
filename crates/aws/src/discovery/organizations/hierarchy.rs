use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn collect_hierarchy(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
) -> Hierarchy {
    let roots = list_roots(client, sdk).await;
    let traversal = stream::unfold(Some(HierarchyCursor::new(roots)), |state| async move {
        let mut cursor = state?;
        if let Err(error) = ensure_running(client.executor) {
            cursor
                .result
                .coverage
                .fail("organizations:Hierarchy", None, &error);
            return Some((Some(cursor.result), None));
        }
        if cursor.pending.is_empty() {
            return Some((Some(cursor.result), None));
        }
        if cursor.visited >= MAX_PARENT_VISITS {
            cursor.result.coverage.fail(
                "organizations:Hierarchy",
                None,
                &data_error("hierarchy parent visit bound exceeded"),
            );
            return Some((Some(cursor.result), None));
        }
        let parent = cursor.pending.pop_front()?;
        let (units, members) = tokio::join!(
            list_units(
                client,
                sdk,
                &parent,
                MAX_DISCOVERED_UNITS.saturating_sub(cursor.result.units.len())
            ),
            list_members(
                client,
                sdk,
                &parent.id,
                MAX_DISCOVERED_ACCOUNTS.saturating_sub(cursor.result.memberships.len())
            )
        );
        cursor.accept(&parent, units, members);
        Some((None, Some(cursor)))
    });
    traversal
        .filter_map(future::ready)
        .fold(None, |_, result| future::ready(Some(result)))
        .await
        .unwrap_or_else(|| {
            let mut coverage = DiscoveryCoverage::success();
            coverage.fail(
                "organizations:Hierarchy",
                None,
                &data_error("hierarchy has no terminal outcome"),
            );
            Hierarchy {
                roots: Vec::new(),
                units: Vec::new(),
                memberships: Vec::new(),
                coverage,
            }
        })
}
