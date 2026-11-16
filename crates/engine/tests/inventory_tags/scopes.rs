use super::{BinaryStore, StoreError, SweepId, Tag, TagCoverage, TagObservation, TestResult, acknowledge, state, support};
use inventory_core::inventory::{AccountId, Observation, OperationId, Region, ResourceKey, ScopeKey, ServiceId};
use inventory_core::record::Exclusions;
use inventory_engine::inventory::WorkSpec;

fn key(account: &str, region: &str, id: &str) -> TestResult<ResourceKey> {
    let mut key = support::resource(id)?;
    key.account = AccountId::parse(account)?;
    key.region = Region::parse(region)?;
    Ok(key)
}

fn work(sweep: &str, key: &ResourceKey) -> TestResult<WorkSpec> {
    Ok(WorkSpec::new(SweepId::new(sweep.to_owned())?, ScopeKey::new(
        key.account.clone(), key.region.clone(), ServiceId::parse("example")?,
        OperationId::parse("ListThings")?, None,
    )?, None, vec![1, 2])?)
}

#[tokio::test]
async fn same_region_kind_id_across_accounts_and_global_scopes_never_leak_or_replace_tags() -> TestResult {
    // Given independent account-only and global/regional collisions with identical pair text.
    let (directory, store, _) = support::fixture().await?;
    let fixtures = [
        (key("123456789012", "us-east-1", "one")?, vec![("owner", "alice"), ("team", "alice")]),
        (key("000000000000", "us-east-1", "one")?, vec![("owner", "alice")]),
        (key("123456789012", "global", "one")?, vec![("owner", "alice")]),
        (key("000000000000", "global", "one")?, vec![("owner", "alice")]),
        (key("123456789012", "us-east-1", "two")?, vec![("owner", "alice")]),
    ];
    // Same account/region items share a page so a complete enumeration does not mark them missing.
    for (index, (account, region)) in [
        ("123456789012", "us-east-1"), ("000000000000", "us-east-1"),
        ("123456789012", "global"), ("000000000000", "global"),
    ].into_iter().enumerate() {
        let sweep = format!("scoped-{index}");
        let lease = state::scoped_work(&store, work(&sweep, &key(account, region, "one")?)?, 100).await?;
        let mut observations = Vec::new();
        let mut tags = Vec::new();
        for (target, pairs) in fixtures.iter().filter(|(target, _)| {
            target.account.as_str() == account && target.region.as_str() == region
        }) {
            observations.push(Observation::new(target.clone(),
                support::observation("one", "literal-record")?.record().clone(),
                &Exclusions::none(), Some(format!("literal-{index}").into()))?);
            tags.push(TagObservation::new(target.clone(), TagCoverage::Complete,
                pairs.iter().map(|(key, value)| Tag::new(*key, *value)).collect::<Result<Vec<_>, _>>()?)?);
        }
        let inserted = u64::try_from(observations.len())?;
        let mut page = support::page(lease, observations, true, None, 100);
        page.tags = tags;
        assert_eq!(store.commit_page(page).await?.stats.resources_inserted, inserted);
    }
    acknowledge(&store).await?;
    let mut retained = store.begin_projection_snapshot().await?;
    for (target, pairs) in &fixtures {
        let expected = pairs.iter().map(|(key, value)| state::pair(key, value)).collect::<Vec<_>>();
        assert_eq!(retained.tags_for(target, None, 1000).await?.tags, expected);
        assert_eq!(state::read_tags(&store, target).await?, expected);
    }

    // When just the selected regional/account target is cleared, the four collisions survive.
    let selected = &fixtures.first().ok_or("selected target missing")?.0;
    state::commit_tags(&store, "clear-one-scope", vec![TagObservation::new(selected.clone(), TagCoverage::Complete, vec![])?], 200).await?;
    for (target, pairs) in &fixtures {
        let expected = pairs.iter().map(|(key, value)| state::pair(key, value)).collect::<Vec<_>>();
        assert_eq!(retained.tags_for(target, None, 1000).await?.tags, expected);
        let latest = if target == selected { vec![] } else { expected };
        assert_eq!(state::read_tags(&store, target).await?, latest);
    }
    retained.finish().await?;
    let pending = store.drain_projection(100).await?;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending.first().and_then(|item| item.key.as_ref()), Some(selected));

    // A persisted foreign-account collision cannot be written through the selected lease.
    let lease = state::scoped_work(&store, support::child_work("foreign-account", selected)?, 300).await?;
    let pool = state::pool(&directory).await?;
    let before = state::database_state(&pool).await?;
    let foreign = key("000000000000", "us-east-1", "one")?;
    let mut invalid = support::page(lease.clone(), vec![], true, None, 300);
    invalid.tags = vec![TagObservation::new(foreign, TagCoverage::Complete, vec![])?];
    assert!(matches!(store.commit_page(invalid).await,
        Err(StoreError::Invalid("page contains data outside its leased account and region"))));
    assert_eq!(state::database_state(&pool).await?, before);
    store.commit_page(support::page(lease, vec![], true, None, 300)).await?;
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    for (target, pairs) in &fixtures {
        let expected = if target == selected { vec![] } else {
            pairs.iter().map(|(key, value)| state::pair(key, value)).collect()
        };
        assert_eq!(state::read_tags(&store, target).await?, expected);
    }
    Ok(())
}
