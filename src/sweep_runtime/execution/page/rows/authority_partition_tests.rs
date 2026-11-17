use super::*;

fn projected_work(sweep: &SweepId, ordinal: u32, branch: Option<&str>, volatile: &str) -> TestResult<WorkSpec> {
    let mut request = BTreeMap::from([(Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned()))]);
    if let Some(branch) = branch { request.insert(Key::Field(FieldId::new(11)?), Value::Text(branch.to_owned())); }
    let bound = Record::encode(&Value::Map(request))?;
    let projected = |namespace: &str| -> TestResult<Record> {
        Ok(Record::encode(&Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(3)?), Value::Text(format!("{namespace}-{volatile}"))),
        ])))?)
    };
    let path = vec![vec![FieldId::new(3)?].into_boxed_slice()].into_boxed_slice();
    let projection = ContextProjectionPlan { selected: path.clone(), response: path.clone(), request: path };
    let context = DiscoveryContext::empty().push(DiscoveryFrame::project(
        ServiceId::parse("workspaces-web")?, OperationId::parse("ListPortals")?, ordinal,
        &projection, &projected("selected")?, &projected("response")?, &projected("request")?,
    )?)?;
    let scope = ScopeKey::new(AccountId::parse("123456789012")?, Region::parse("us-east-1")?, ServiceId::parse("workspaces-web")?, OperationId::parse("GetPortal")?, None)?;
    let scope = if let Some(branch) = branch { scope.with_branch(branch.to_owned())? } else { scope };
    Ok(WorkSpec::new(sweep.clone(), scope.with_context(context.discriminator()?), None, bound.into_bytes())?.with_context(context.encode()?)?)
}

async fn projected_page(store: &BinaryStore, work: WorkSpec, observations: Vec<Observation>, at: i64) -> TestResult<Vec<u8>> {
    let sweep = work.sweep().clone();
    store.seed_work(work).await?;
    let lease = store.claim(&sweep, at, 100).await?.ok_or("projected work absent")?;
    let bindings = lease.bindings().to_vec();
    page(store, lease, observations, Vec::new(), at + 1).await?;
    Ok(bindings)
}

#[tokio::test]
async fn given_equal_bindings_with_different_unbound_selected_response_and_request_values_when_completed_then_they_share_one_authority_and_two_misses_retire_it() -> TestResult {
    // Given a real projected frame with distinct values in all three namespaces.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    let first = projected_work(&SweepId::new("projection-one".to_owned())?, 1, None, "old")?;
    let first_scope = first.scope().clone();
    let bindings = projected_page(&store, first, vec![portal_observation(&catalog, "GetPortal", PORTAL, "stable")?], 1).await?;
    let second = projected_work(&SweepId::new("projection-two".to_owned())?, 9, None, "new")?;
    assert_ne!(first_scope.context(), second.scope().context());
    // When unrelated projected values and ordinal change, only execution identity changes.
    assert_eq!(projected_page(&store, second, vec![portal_observation(&catalog, "GetPortal", PORTAL, "stable")?], 3).await?, bindings);
    assert_eq!(ownership(&directory).await?, (1, 0));
    let seen = first_seen(&directory).await?;
    let third = projected_work(&SweepId::new("projection-miss-one".to_owned())?, 18, None, "third")?;
    assert_eq!(projected_page(&store, third, Vec::new(), 5).await?, bindings);
    assert_eq!(ownership(&directory).await?, (1, 1));
    assert_eq!(first_seen(&directory).await?, seen);
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let fourth = projected_work(&SweepId::new("projection-miss-two".to_owned())?, 27, None, "fourth")?;
    assert_eq!(projected_page(&reopened, fourth, Vec::new(), 7).await?, bindings);
    // Then no volatile historical owner remains to make the resource immortal.
    assert_eq!(reopened.projection_resource(&key()?).await?, None);
    assert_eq!(reopened.projection_resources(None, 100).await?, Vec::new());
    Ok(())
}

fn branch_observation(catalog: &RuntimeCatalog<'_>, name: &str, metadata: Option<&str>) -> TestResult<Observation> {
    let plan = catalog.lookup("workspaces-web", "GetPortal").ok_or("branch plan absent")?;
    let mut fields = BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned())),
        (Key::Field(FieldId::new(3)?), Value::Text(name.to_owned())),
    ]);
    if let Some(metadata) = metadata { fields.insert(Key::Field(FieldId::new(7)?), Value::Text(metadata.to_owned())); }
    Ok(Observation::new(key()?, Record::encode(&Value::Map(fields))?, &Exclusions::none(), Some(name.into()))?.with_facet_precedence(plan.facet_precedence()))
}

#[tokio::test]
async fn given_two_branches_owning_the_same_resource_when_one_retires_then_only_its_facet_disappears_in_either_completion_order() -> TestResult {
    for reverse in [false, true] {
        // Given two branch-specific facets for exactly the same canonical ResourceKey and operation.
        let bytes = fixture::compiled_branched()?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let (directory, store) = store(&catalog).await?;
        let mut owners = [("alpha", "branch-A", Some("alpha-metadata")), ("omega", "branch-B", None)];
        if reverse { owners.reverse(); }
        for ((branch, name, metadata), at) in owners.into_iter().zip([1, 3]) {
            let work = projected_work(&SweepId::new(format!("branch-{at}"))?, 1, Some(branch), "same")?;
            projected_page(&store, work, vec![branch_observation(&catalog, name, metadata)?], at).await?;
        }
        assert_eq!(reopened_name(&store).await?, "branch-B");
        assert_eq!(field(&state(&directory, &catalog).await?.0, 7)?, Some(Value::Text("alpha-metadata".to_owned())));
        assert_eq!(ownership(&directory).await?, (2, 0));
        let seen = first_seen(&directory).await?;
        // When only alpha completes absent twice, omega remains an independent live owner.
        projected_page(&store, projected_work(&SweepId::new("alpha-miss-one".to_owned())?, 7, Some("alpha"), "changed")?, Vec::new(), 5).await?;
        assert_eq!(field(&state(&directory, &catalog).await?.0, 7)?, Some(Value::Text("alpha-metadata".to_owned())));
        drop(store);
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        projected_page(&reopened, projected_work(&SweepId::new("alpha-miss-two".to_owned())?, 8, Some("alpha"), "again")?, Vec::new(), 7).await?;
        // Then exact canonical values, first_seen, and the surviving facet prove there was no cascade or overwrite.
        assert_eq!(reopened_name(&reopened).await?, "branch-B");
        assert_eq!(field(&state(&directory, &catalog).await?.0, 3)?, Some(Value::Text("branch-B".to_owned())));
        assert_eq!(field(&state(&directory, &catalog).await?.0, 7)?, None);
        assert_eq!(first_seen(&directory).await?, seen);
        let db = sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?;
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE id=?1").bind(PORTAL).fetch_one(&db).await?, 1);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_scopes WHERE id=?1 AND missing_complete=0").bind(PORTAL).fetch_one(&db).await?, 1);
        db.close().await;
    }
    Ok(())
}

async fn reopened_name(store: &BinaryStore) -> TestResult<String> {
    Ok(store.projection_resource(&key()?).await?.ok_or("branch resource absent")?.name.ok_or("branch name absent")?)
}
