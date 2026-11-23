use super::*;
#[tokio::test]
async fn ga_request_parent_relations_reject_hostile_scope_and_type_without_selected_parent_fields()
-> TestResult {
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("eu-west-1")?;
    let sweep = SweepId::new("request-origin-hostile-scope".to_owned())?;
    for (root_operation, child_operation) in [
        ("ListAccelerators", "ListListeners"),
        (
            "ListCustomRoutingAccelerators",
            "ListCustomRoutingListeners",
        ),
    ] {
        // Given real compiled discovery, a durable root lease and actual bound request.
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        let parent = fixture::plan(&catalog, "global-accelerator", root_operation)?;
        let child = fixture::plan(&catalog, "global-accelerator", child_operation)?;
        let parent_arn = "arn:aws:globalaccelerator::123456789012:accelerator/parent";
        let scope = ScopeKey::new(
            account.clone(),
            Region::Global,
            ServiceId::parse(parent.service_name())?,
            OperationId::parse(root_operation)?,
            None,
        )?;
        let bindings = input::root_bindings(parent, Some(&account), Some(&Region::Global))?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
        let root_lease = store
            .claim(&sweep, 1000, 100_000)
            .await?
            .ok_or("root lease absent")?;
        let root_request = actual_request(parent, &root_lease)?;
        let response = fixture::decode(
            parent,
            &serde_json::to_vec(&json!({"Accelerators":[{"AcceleratorArn":parent_arn}]}))?,
        )?;
        let context = Context::new(parent, &sweep, &account, &home, &consumers);
        let ancestry = crate::execution::page::preparation::ancestry(
            root_lease.context(),
            root_lease.scope().context(),
        )?;
        let (parent_observations, children) = project_children(
            &catalog,
            &context,
            root_lease.scope(),
            &root_request,
            &response,
            &ancestry,
        )?;
        assert!(
            children
                .iter()
                .any(|work| work.scope().operation().as_str() == child_operation),
            "real child discovery did not schedule listener enumeration"
        );
        store
            .commit_page(PageCommit {
                lease: root_lease,
                observations: parent_observations,
                edges: Vec::new(),
                unresolved: Vec::new(),
                tags: Vec::new(),
                children,
                next_token: None,
                continuation_digest: None,
                done: true,
                observed_at_ms: 1001,
            })
            .await?;
        // Claim generated work, rather than exposing private WorkSpec binding/context fields.
        let lease = loop {
            let lease = store
                .claim(&sweep, 1002, 100_000)
                .await?
                .ok_or("generated listener lease absent")?;
            if lease.scope().operation().as_str() == child_operation {
                break lease;
            }
        };
        let request = actual_request(child, &lease)?;
        let ancestry = crate::execution::page::preparation::ancestry(
            lease.context(),
            lease.scope().context(),
        )?;
        let response = fixture::decode(
            child,
            &serde_json::to_vec(&json!({"Listeners":[{
                "ListenerArn":"arn:aws:globalaccelerator::123456789012:accelerator/parent/listener/child"
            }]}))?,
        )?;
        let context = Context::new(child, &sweep, &account, &home, &consumers);
        let (observations, children) = project_children(
            &catalog,
            &context,
            lease.scope(),
            &request,
            &response,
            &ancestry,
        )?;
        let rows = record::item_records(&response, child.items())?;
        let rule = catalog
            .relationships()
            .iter()
            .find(|rule| {
                rule.source_kind == *child.kind() && rule.source == RelationSource::Request
            })
            .ok_or("reviewed request-origin listener relationship absent")?;
        assert!(child.supports_relationship(rule)?);
        let request_region =
            routing::request_region(child, lease.scope().region(), &home, lease.scope().branch())?;
        let valid = references(
            &catalog,
            child,
            &request_region,
            &rows,
            &observations,
            &request,
            lease.scope().parent(),
        )?;
        assert_eq!(valid.len(), 1);
        assert_eq!(
            valid
                .first()
                .ok_or("valid request parent reference absent")?
                .target
                .value()
                .as_str(),
            parent_arn
        );
        for observation in &observations {
            assert!(record::path_value(observation.record().root(), &rule.source_path)?.is_none());
            assert!(
                matches!(record::path_value(request.root(), &rule.source_path)?,
                Some(ValueRef::Text(value)) if value == parent_arn)
            );
        }
        // When only the actual bound parent request is hostile, no selected output may rescue it.
        for hostile in [
            Value::Text("arn:aws:globalaccelerator::999999999999:accelerator/foreign".to_owned()),
            Value::Text(
                "arn:aws-cn:globalaccelerator::123456789012:accelerator/foreign".to_owned(),
            ),
            Value::Text(
                "arn:aws:globalaccelerator:us-east-1:123456789012:accelerator/foreign".to_owned(),
            ),
            Value::Text("arn:broken".to_owned()),
            Value::Int(7),
        ] {
            let hostile_request =
                replace_bound_value(&request, &rule.source_path, hostile.clone())?;
            assert_eq!(
                record::path_value(hostile_request.root(), &rule.source_path)?
                    .map(|value| value.to_value())
                    .transpose()?,
                Some(hostile)
            );
            assert!(
                references(
                    &catalog,
                    child,
                    &request_region,
                    &rows,
                    &observations,
                    &hostile_request,
                    lease.scope().parent()
                )
                .is_err()
            );
        }
        // Then the same valid reference survives checked persistence and resolves to the discovered parent.
        let key = observations
            .first()
            .ok_or("listener observation absent")?
            .key()
            .clone();
        store
            .commit_page(PageCommit {
                lease,
                observations,
                unresolved: valid,
                children,
                edges: Vec::new(),
                tags: Vec::new(),
                next_token: None,
                continuation_digest: None,
                done: true,
                observed_at_ms: 1003,
            })
            .await?;
        let edges = store.projection_edges_for(&key, None, 10).await?;
        assert_eq!(edges.edges.len(), 1);
        let edge = edges
            .edges
            .first()
            .ok_or("persisted parent relation absent")?;
        assert_eq!(edge.from, key);
        assert_eq!(edge.relation, rule.relation);
        assert_eq!(edge.to.kind, rule.target_kind);
        assert_eq!(edge.to.id.as_str(), parent_arn);
        assert_eq!(edge.to.account, account);
        assert_eq!(edge.to.region, Region::Global);
    }
    Ok(())
}
