use super::*;
#[tokio::test]
async fn given_large_valid_ancestry_and_many_small_rows_when_prepared_then_aggregate_overload_leaves_page_and_cursor_atomic()
-> TestResult {
    use inventory_core::{
        catalog::runtime::ContextProjectionPlan,
        inventory::{DiscoveryFrame, ScopeOutcome},
        record::{FieldId, Key},
    };
    let scenario = Scenario::new("aggregate-page-context-overload").await?;
    let root = scenario.lease().await?;
    let root_scope = root.scope().clone();
    scenario
        .store
        .finish_scope(root, ScopeOutcome::Partial, now_ms()?, None)
        .await?;
    let field = FieldId::new(1)?;
    let large = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(field),
        Value::Text("x".repeat(512 * 1024)),
    )])))?;
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan {
        selected: vec![vec![field].into_boxed_slice()].into_boxed_slice(),
        response: Box::new([]),
        request: Box::new([]),
    };
    let ancestry = DiscoveryContext::empty().push(DiscoveryFrame::project(
        ServiceId::parse("transfer")?,
        OperationId::parse("ListServers")?,
        0,
        &projection,
        &large,
        &empty,
        &empty,
    )?)?;
    let scope = root_scope.with_context(ancestry.discriminator()?);
    scenario
        .store
        .seed_work(
            WorkSpec::new(scenario.sweep.clone(), scope, None, Vec::new())?
                .with_context(ancestry.encode()?)?,
        )
        .await?;
    let lease = scenario.lease().await?;
    let generation = lease.generation();
    let plan = fixture::plan(&scenario.catalog, "iottwinmaker", "ListWorkspaces")?;
    let consumers = ConsumerIndex::build(&scenario.catalog)?;
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let context = Context {
        catalog: &scenario.catalog,
        account: &scenario.account,
        home: &scenario.home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 10,
    };
    let request = scenario.request()?;
    let result = prepare(
        &context,
        &lease,
        plan,
        cursor::Cursor::decode(None)?,
        scenario.response(128)?,
        &request,
        &ancestry,
    )
    .await;
    assert!(matches!(
        result,
        Err(SweepError::Operation(
            "page discovery contexts exceed the aggregate byte bound"
        ))
    ));
    scenario.assert_atomic().await?;
    let after_expiry = now_ms()?
        .checked_add(3_600_001)
        .ok_or("test timestamp overflow")?;
    let resumed = scenario
        .store
        .claim(&scenario.sweep, after_expiry, 3_600_000)
        .await?
        .ok_or("rejected page ownership disappeared")?;
    assert!(
        resumed.token().is_none(),
        "rejected page advanced its durable cursor"
    );
    assert!(resumed.generation() > generation);
    assert_eq!(resumed.context(), ancestry.encode()?);
    Ok(())
}
