use super::*;
use inventory_core::record::Exclusions;

async fn decoy_page(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    key: &ResourceKey,
    record: Option<Record>,
    at: i64,
) -> TestResult {
    let plan = catalog
        .lookup("oam", "ListSinks")
        .ok_or("OAM sink plan absent")?;
    let observations = record
        .map(|record| {
            Observation::new(key.clone(), record, &Exclusions::none(), None)
                .map(|observation| observation.with_facet_precedence(plan.facet_precedence()))
        })
        .transpose()?
        .into_iter()
        .collect();
    let sweep = SweepId::new(format!("oam-decoy-{at}"))?;
    let scope = ScopeKey::new(
        key.account.clone(),
        key.region.clone(),
        ServiceId::parse("oam")?,
        OperationId::parse("ListSinks")?,
        None,
    )?;
    let request = Record::encode(&Value::Map(BTreeMap::new()))?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            request.into_bytes(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("decoy lease absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            unresolved: Vec::new(),
            edges: Vec::new(),
            tags: Vec::new(),
            children: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_foreign_oam_sink_reference_when_equal_value_wrong_owner_or_region_target_changes_and_disappears_then_only_exact_namespace_stays_resolved()
-> TestResult {
    let bytes = oam_fixture::compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("oam", "ListSinks")
        .ok_or("OAM sink plan absent")?;
    let record = |arn: &str| -> TestResult<Record> {
        oam_fixture::rows(
            plan,
            json!({"Items":[{"Arn":arn,"Id":"00000000-0000-0000-0000-000000000001"}]}),
        )?
        .into_iter()
        .next()
        .ok_or_else(|| "sink record absent".into())
    };
    for wrong_account in [false, true] {
        let (directory, store) = store(&catalog).await?;
        persist_oam(&store, &catalog, "ListLinks", "123456789012", 1).await?;
        persist_oam(&store, &catalog, "ListSinks", "210987654321", 2).await?;
        let decoy = ResourceKey {
            account: AccountId::parse(if wrong_account {
                "123456789012"
            } else {
                "210987654321"
            })?,
            region: Region::parse(if wrong_account {
                "us-east-1"
            } else {
                "us-west-2"
            })?,
            kind: inventory_core::inventory::Kind::parse("AWS::Oam::Sink")?,
            id: ResourceId::parse(SINK)?,
        };
        let other = SINK.replace("000000000001", "000000000003");
        for (index, value) in [Some(record(SINK)?), Some(record(&other)?), None, None]
            .into_iter()
            .enumerate()
        {
            decoy_page(
                &store,
                &catalog,
                &decoy,
                value,
                i64::try_from(index)?
                    .checked_add(3)
                    .ok_or("time overflow")?,
            )
            .await?;
            assert_eq!(
                store.projection_resource(&decoy).await?.is_some(),
                index < 3
            );
            let edges = store.projection_edges(None, 100).await?.edges;
            assert_eq!(
                edges.len(),
                1,
                "wrong_account={wrong_account}, stage={index}"
            );
            let edge = edges.first().ok_or("intended OAM edge disappeared")?;
            assert_eq!(edge.from.account.as_str(), "123456789012");
            assert_eq!(edge.from.region.as_str(), "us-east-1");
            assert_eq!(edge.from.id.as_str(), LINK);
            assert_eq!(edge.to.account.as_str(), "210987654321");
            assert_eq!(edge.to.region.as_str(), "us-east-1");
            assert_eq!(edge.to.id.as_str(), SINK);
            assert_eq!(edge.to.kind.as_str(), "AWS::Oam::Sink");
            assert_eq!(edge.relation.as_str(), "TARGETS_SINK");
            assert_eq!(
                edge.source,
                inventory_core::inventory::EdgeSource::ReviewedRule.as_str()
            );
            let pool = sqlx::SqlitePool::connect(&format!(
                "sqlite://{}",
                directory.path().join("inventory.sqlite").display()
            ))
            .await?;
            assert_eq!(
                sqlx::query_scalar::<_, String>(
                    "SELECT resolution FROM unresolved_refs WHERE from_id=?1"
                )
                .bind(LINK)
                .fetch_one(&pool)
                .await?,
                "resolved"
            );
            pool.close().await;
        }
    }
    Ok(())
}
