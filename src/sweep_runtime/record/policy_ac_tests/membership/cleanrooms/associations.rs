use super::*;

fn table_arn(id: &str) -> String { format!("arn:aws:cleanrooms:us-east-1:123456789012:configuredtable/{id}") }
fn membership_arn(id: &str) -> String { format!("arn:aws:cleanrooms:us-east-1:123456789012:membership/{id}") }

#[tokio::test]
async fn given_two_memberships_tables_models_and_namespaces_when_association_pages_resume_then_each_persisted_edge_keeps_both_exact_endpoints() -> TestResult {
    // Given actual pinned producers from all three services and independently specified association endpoints.
    let association_ops = ["ListConfiguredTableAssociations", "ListConfiguredAudienceModelAssociations", "ListIdNamespaceAssociations"];
    let cleanrooms_ops = ["ListMemberships", "ListConfiguredTables", "ListConfiguredTableAssociations", "ListConfiguredAudienceModelAssociations", "ListIdNamespaceAssociations"];
    let compiled = super::super::super::fixture::compile_many(&[
        ("cleanrooms", "2022-02-17", &cleanrooms_ops),
        ("cleanroomsml", "2023-09-06", &["ListConfiguredAudienceModels"]),
        ("entityresolution", "2018-05-10", &["ListIdNamespaces"]),
    ])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let cases = SPEC.iter().filter(|spec| association_ops.contains(&spec.0)).map(|spec| case(*spec)).collect::<TestResult<Vec<_>>>()?;
    assert_eq!(cases.len(), 3);
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new("cleanrooms-dual-association-endpoints".to_owned())?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("associations.sqlite");
    let mut store = BinaryStore::create(&path, SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?).await?;
    let mut expected_resources = BTreeSet::new();
    let mut expected_edges = BTreeSet::new();
    for owner in OWNERS { expected_resources.insert(("AWS::CleanRooms::Membership".to_owned(), membership_arn(owner))); }
    for table in TABLES { expected_resources.insert(("AWS::CleanRooms::ConfiguredTable".to_owned(), table_arn(table))); }
    for model in MODELS { expected_resources.insert(("AWS::ApiOnly::CleanRoomsMLConfiguredAudienceModel".to_owned(), model.to_owned())); }
    for namespace in NAMESPACES { expected_resources.insert(("AWS::EntityResolution::IdNamespace".to_owned(), namespace.to_owned())); }
    for case in &cases {
        for owner in &case.owners {
            let (table, model) = endpoint_ids(&owner.value)?;
            let (target_kind, target_id, relation) = if case.operation == "ListConfiguredTableAssociations" {
                ("AWS::CleanRooms::ConfiguredTable", table_arn(table), "USES_CONFIGURED_TABLE")
            } else if case.operation == "ListConfiguredAudienceModelAssociations" {
                ("AWS::ApiOnly::CleanRoomsMLConfiguredAudienceModel", model.to_owned(), "USES_CONFIGURED_AUDIENCE_MODEL")
            } else { ("AWS::EntityResolution::IdNamespace", namespace_arn(&owner.value)?.to_owned(), "USES_ID_NAMESPACE") };
            for page in &owner.pages {
                expected_resources.insert((case.kind.to_owned(), page.id.clone()));
                expected_edges.insert((case.kind.to_owned(), page.id.clone(), "AWS::CleanRooms::Membership".to_owned(), membership_arn(&owner.value), "IN_MEMBERSHIP".to_owned()));
                expected_edges.insert((case.kind.to_owned(), page.id.clone(), target_kind.to_owned(), target_id.clone(), relation.to_owned()));
            }
        }
    }
    let expected_resources = expected_resources.into_iter().map(|(kind, id)| resource_key(&account, &region, &kind, &id))
        .collect::<TestResult<BTreeSet<_>>>()?;
    for (service, operation) in [("cleanrooms", "ListMemberships"), ("cleanrooms", "ListConfiguredTables"), ("cleanroomsml", "ListConfiguredAudienceModels"), ("entityresolution", "ListIdNamespaces")] {
        let plan = catalog.lookup(service, operation).ok_or("association endpoint producer absent")?;
        let scope = ScopeKey::new(account.clone(), region.clone(), ServiceId::parse(service)?, OperationId::parse(operation)?, None)?;
        store.seed_work(WorkSpec::new(sweep.clone(), scope, None, input::root_bindings(plan, Some(&account), Some(&region))?.into_bytes())?).await?;
    }
    // When real root and resumed child requests decode native responses and atomically commit resources and joins.
    let mut child_pages = 0_u8;
    for at in 0..16 {
        let lease = store.claim(&sweep, at, 10_000).await?.ok_or("expected association fixture page absent")?;
        let plan = catalog.lookup(lease.scope().service().as_str(), lease.scope().operation().as_str()).ok_or("leased association plan absent")?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        let mut is_association = false;
        let body = match plan.operation_name() {
            "ListMemberships" => {
                NativeRequest::get("/memberships".to_owned(), &[]).assert_matches(&wire)?;
                json!({"membershipSummaries":OWNERS.map(membership)})
            },
            "ListConfiguredTables" => {
                NativeRequest::get("/configuredTables".to_owned(), &[]).assert_matches(&wire)?;
                json!({"configuredTableSummaries":TABLES.map(|id| json!({"id":id,"arn":table_arn(id),"name":"same",
                    "createTime":1767225600,"updateTime":1767312000,"analysisRuleTypes":["AGGREGATION"],"analysisMethod":"DIRECT_QUERY"}))})
            },
            "ListConfiguredAudienceModels" => {
                NativeRequest::get("/configured-audience-model".to_owned(), &[]).assert_matches(&wire)?;
                json!({"configuredAudienceModels":MODELS.map(|arn| json!({"configuredAudienceModelArn":arn,"name":"same","status":"ACTIVE",
                    "createTime":"2026-01-01T00:00:00Z","updateTime":"2026-01-02T00:00:00Z",
                    "audienceModelArn":"arn:aws:cleanrooms-ml:us-east-1:123456789012:audience-model/trained",
                    "outputConfig":{"roleArn":"arn:aws:iam::123456789012:role/metadata-output","destination":{"s3Destination":{"s3Uri":"s3://metadata-output/prefix"}}}}))})
            },
            "ListIdNamespaces" => {
                NativeRequest::get("/idnamespaces".to_owned(), &[]).assert_matches(&wire)?;
                json!({"idNamespaceSummaries":NAMESPACES.into_iter().zip(["namespace-a", "namespace-b"]).map(|(arn, name)| json!({
                    "idNamespaceName":name,"idNamespaceArn":arn,"type":"SOURCE","createdAt":1767225600,"updatedAt":1767312000})).collect::<Vec<_>>()})
            },
            operation => {
                is_association = true;
                child_pages = child_pages.checked_add(1).ok_or("association page count overflow")?;
                let case = cases.iter().find(|case| case.operation == operation).ok_or("unreviewed association child")?;
                let owner_id = text(&actual, &compiled.fields, "membershipIdentifier")?;
                let owner = case.owners.iter().find(|owner| owner.value == owner_id).ok_or("association lost its membership")?;
                let page = owner.pages.get(usize::from(lease.token().is_some())).ok_or("association page fixture absent")?;
                page.wire.assert_matches(&wire)?;
                page.body.clone()
            },
        };
        let response = decode(plan, &body)?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let (observations, children) = child::policy_dfgk_tests::project_children(&catalog, &context, lease.scope(), &actual, &response, &ancestry)?;
        let unresolved = if is_association {
            assert_eq!(observations.len(), 1);
            assert!(children.is_empty());
            let selected = record::item_records(&response, plan.items())?;
            let references = rows::references(&catalog, plan, &home, &selected, &observations, &actual, None)?;
            assert_eq!(references.len(), 2, "both membership and selected endpoint must survive projection");
            references
        } else { Vec::new() };
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| (Some(next.cursor.into_bytes()), next.continuation_digest));
        store.commit_page(PageCommit { lease, observations, children, unresolved, edges: Vec::new(), tags: Vec::new(), next_token, continuation_digest, done, observed_at_ms: at }).await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then SQLite reopen proves exact source, destination, relation, authority and scope, not merely edge counts.
    assert_eq!(child_pages, 12);
    assert!(store.claim(&sweep, 17, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(resources.len(), expected_resources.len());
    let actual_resources = resources.into_iter().map(|resource| resource.key).collect::<BTreeSet<_>>();
    assert_eq!(actual_resources, expected_resources);
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), expected_edges.len());
    let actual_edges = edges.edges.into_iter().map(|edge| {
        assert_eq!(edge.source, "reviewed-rule");
        assert_eq!(edge.from.account, account);
        assert_eq!(edge.to.account, account);
        assert_eq!(edge.from.region, region);
        assert_eq!(edge.to.region, region);
        (edge.from.kind.to_string(), edge.from.id.to_string(), edge.to.kind.to_string(), edge.to.id.to_string(), edge.relation.to_string())
    }).collect::<BTreeSet<_>>();
    assert_eq!(actual_edges, expected_edges);
    Ok(())
}
