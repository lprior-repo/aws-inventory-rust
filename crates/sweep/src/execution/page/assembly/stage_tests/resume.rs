use super::*;
#[tokio::test]
async fn given_native_proton_stages_when_rooted_projected_and_reopened_per_page_then_only_minors_emit_and_each_cursor_keeps_its_real_parent()
-> TestResult {
    let bytes = crate::record::policy_lp_tests::fixture::compiled(
        "proton",
        "2020-07-20",
        &["ListServiceTemplates", "ListServiceTemplateVersions"],
        TEMPLATE_KIND,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("staged.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&database, registry).await?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let sweep = SweepId::new("production-proton-stages".to_owned())?;
    store
        .seed_works(&root_work(&catalog, &account, &home, &sweep)?)
        .await?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let executor = inventory_engine::limiter::ApiExecutor::new(Default::default())?;
    let context = Context {
        catalog: &catalog,
        account: &account,
        home: &home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 10,
    };
    let mut seen = BTreeSet::new();
    for _ in 0..16 {
        let Some(lease) = store.claim(&sweep, now_ms()?, 3_600_000).await? else {
            break;
        };
        let plan = catalog
            .lookup(
                lease.scope().service().as_str(),
                lease.scope().operation().as_str(),
            )
            .ok_or("persisted reviewed plan absent")?;
        let (request, wire) = wire_request(plan, &lease, &home)?;
        assert!(
            seen.insert((
                plan.operation_name().to_owned(),
                lease.scope().branch().map(str::to_owned),
                wire.get("templateName")
                    .and_then(Json::as_str)
                    .map(str::to_owned),
                wire.get("majorVersion")
                    .and_then(Json::as_str)
                    .map(str::to_owned),
                lease.token().is_some()
            )),
            "duplicate or cross-stage work admitted"
        );
        let body = native_page(plan, &lease, &wire)?;
        apply_page(&store, &context, plan, lease, request, body).await?;
        drop(store);
        store = BinaryStore::open_with_catalog(&database, &bytes).await?;
    }
    assert_eq!(seen.len(), 14);
    let summary = store.sweep_summary(&sweep).await?;
    assert_eq!(summary.work_pending, 0);
    assert_eq!(summary.complete_scopes, 7);
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", database.display())).await?;
    let versions: BTreeSet<String> = sqlx::query_scalar("SELECT id FROM resources WHERE kind=?1")
        .bind(VERSION_KIND)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .collect();
    let expected: BTreeSet<String> = ["one", "two"]
        .into_iter()
        .flat_map(|name| {
            ["1", "2"].into_iter().flat_map(move |major| {
                ["1", "2"]
                    .into_iter()
                    .map(move |minor| format!("{}:{major}.{minor}", template_arn(name)))
            })
        })
        .collect();
    assert_eq!(
        versions, expected,
        "major latest-minor summaries are discovery context, not resource observations"
    );
    let parents: Vec<(String, String)> =
        sqlx::query_as("SELECT parent_id,child_id FROM resource_parents WHERE child_kind=?1")
            .bind(VERSION_KIND)
            .fetch_all(&pool)
            .await?;
    assert_eq!(parents.len(), 8);
    for (parent, child) in parents {
        assert!(
            child
                .strip_prefix(&parent)
                .is_some_and(|version| version.starts_with(':'))
        );
    }
    pool.close().await;
    Ok(())
}
