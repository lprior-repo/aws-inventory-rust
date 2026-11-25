use super::*;
pub(super) async fn consume_family(family: &Family) -> TestResult {
    // Given: two real parent rows and the embedded, pinned-model-compiled child plans.
    let excluded: &[&str] = match family.service {
        "api-gateway" => &[
            "GetIntegration",
            "GetMethod",
            "GetModel",
            "GetModels",
            "GetExport",
            "GetSdk",
            "PutRestApi",
        ],
        "apigatewayv2" => &["GetIntegrations", "GetModels", "CreateRoute"],
        "batch" => &["DescribeJobDefinitions", "DescribeJobs", "SubmitJob"],
        _ => return Err("unreviewed child fixture family".into()),
    };
    let operations = std::iter::once(family.root)
        .chain(family.children.iter().map(|case| case.operation))
        .chain(excluded.iter().copied())
        .collect::<Vec<_>>();
    let compiled = compile(family.service, family.date, &operations)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for operation in excluded {
        assert!(
            catalog.lookup(family.service, operation).is_none(),
            "content or write plan became callable: {operation}"
        );
    }
    let consumers = ConsumerIndex::build(&catalog)?;
    let parent = catalog
        .lookup(family.service, family.root)
        .ok_or("parent plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new(format!("{}-safe-children", family.service))?;
    let scope = ScopeKey::new(
        account.clone(),
        region.clone(),
        ServiceId::parse(family.service)?,
        OperationId::parse(family.root)?,
        None,
    )?;
    let root_request = input::root_bindings(parent, Some(&account), Some(&region))?;
    let body = json!({family.root_items:[{family.root_key:parent_id(family,"a")},{family.root_key:parent_id(family,"b")}]});
    let response = decode(parent, &body)?;
    let context = child::Context::new(parent, &sweep, &account, &home, &consumers);
    let (observations, children) = child::policy_dfgk_tests::project_children(
        &catalog,
        &context,
        &scope,
        &root_request,
        &response,
        &DiscoveryContext::empty(),
    )?;
    assert_eq!(observations.len(), 2);
    let (root_kind, relation) = match family.service {
        "api-gateway" => ("AWS::ApiGateway::RestApi", "IN_REST_API"),
        "apigatewayv2" => ("AWS::ApiGatewayV2::Api", "IN_API"),
        "batch" => ("AWS::Batch::JobQueue", "IN_JOB_QUEUE"),
        _ => return Err("unreviewed native root kind".into()),
    };
    assert_eq!(parent.kind().as_str(), root_kind);
    let expected_roots = ["a", "b"]
        .into_iter()
        .map(|suffix| resource_key(&account, &region, root_kind, &parent_id(family, suffix)))
        .collect::<TestResult<BTreeSet<_>>>()?;
    assert_eq!(
        observations
            .iter()
            .map(|row| row.key().clone())
            .collect::<BTreeSet<_>>(),
        expected_roots
    );
    let mut expected_resources = expected_roots.clone();
    let mut expected_edges = BTreeSet::new();
    for suffix in ["a", "b"] {
        let owner = parent_id(family, suffix);
        for case in family.children {
            for resumed in [false, true] {
                let native = child_body(family, case, &owner, resumed)?;
                for key in page_keys(case, &owner, &native, &account, &region)? {
                    expected_edges.insert((
                        key.kind.to_string(),
                        key.id.to_string(),
                        root_kind.to_owned(),
                        owner.clone(),
                        relation.to_owned(),
                    ));
                    assert!(
                        expected_resources.insert(key),
                        "duplicate native fixture identity"
                    );
                }
            }
        }
    }
    assert_eq!(
        children.len(),
        family
            .children
            .len()
            .checked_mul(2)
            .ok_or("child count overflow")?
    );
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("native-children.sqlite");
    let mut store = BinaryStore::create(
        &path,
        SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
    )
    .await?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            root_request.into_bytes(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, 0, 10_000)
        .await?
        .ok_or("parent lease absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            children,
            unresolved: Vec::new(),
            edges: Vec::new(),
            tags: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: 0,
        })
        .await?;
    let mut seen = expected_roots;
    // When: durable child requests and their continuation pages are encoded, decoded, projected and committed.
    for at in 1..=32 {
        let Some(lease) = store.claim(&sweep, at, 10_000).await? else {
            break;
        };
        let plan = catalog
            .lookup(family.service, lease.scope().operation().as_str())
            .ok_or("child plan absent")?;
        let case = family
            .children
            .iter()
            .find(|case| case.operation == plan.operation_name())
            .ok_or("unexpected child scheduled")?;
        assert_eq!(plan.kind().as_str(), case.kind);
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let parent_value = text(&base, &compiled.fields, family.input_key)?;
        assert!([parent_id(family, "a"), parent_id(family, "b")].contains(&parent_value));
        assert_eq!(
            lease
                .scope()
                .parent()
                .ok_or("child lost durable parent")?
                .id
                .as_str(),
            parent_value
        );
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        let resumed = lease.token().is_some();
        assert_wire(family, case, &wire, &parent_value, resumed)?;
        if resumed {
            assert_eq!(
                text(
                    &actual,
                    &compiled.fields,
                    if family.token == "position" {
                        "position"
                    } else if family.service == "apigatewayv2" {
                        "NextToken"
                    } else {
                        "nextToken"
                    }
                )?,
                "later"
            );
        }
        let native = child_body(family, case, &parent_value, resumed)?;
        let expected_page = page_keys(case, &parent_value, &native, &account, &region)?;
        let response = decode(plan, &native)?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            lease.scope(),
            &actual,
            &response,
            &ancestry,
        )?;
        assert!(children.is_empty());
        assert_eq!(observations.len(), expected_page.len());
        assert_eq!(
            observations
                .iter()
                .map(|row| row.key().clone())
                .collect::<BTreeSet<_>>(),
            expected_page
        );
        for observation in &observations {
            assert!(
                seen.insert(observation.key().clone()),
                "duplicate projected native key"
            );
        }
        let unresolved = child::policy_dfgk_tests::projected_references(
            &catalog,
            &context,
            lease.scope(),
            &actual,
            &response,
            &observations,
        )?;
        assert_eq!(unresolved.len(), observations.len());
        assert!(
            unresolved
                .iter()
                .all(|reference| reference.target.value().as_str() == parent_value)
        );
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        assert_eq!(next.is_some(), !resumed);
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| {
            (Some(next.cursor.into_bytes()), next.continuation_digest)
        });
        store
            .commit_page(PageCommit {
                lease,
                observations,
                children,
                unresolved,
                edges: Vec::new(),
                tags: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: at,
            })
            .await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then: both parents retain every distinct canonical child, including later pages and all seven Batch statuses.
    assert!(store.claim(&sweep, 33, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(seen, expected_resources);
    assert_eq!(resources.len(), expected_resources.len());
    assert_eq!(
        resources
            .into_iter()
            .map(|resource| resource.key)
            .collect::<BTreeSet<_>>(),
        expected_resources
    );
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), expected_edges.len());
    let actual_edges = edges
        .edges
        .into_iter()
        .map(|edge| {
            assert_eq!(edge.source, "reviewed-rule");
            assert_eq!(edge.from.account, account);
            assert_eq!(edge.to.account, account);
            assert_eq!(edge.from.region, region);
            assert_eq!(edge.to.region, region);
            (
                edge.from.kind.to_string(),
                edge.from.id.to_string(),
                edge.to.kind.to_string(),
                edge.to.id.to_string(),
                edge.relation.to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_edges, expected_edges);
    Ok(())
}
