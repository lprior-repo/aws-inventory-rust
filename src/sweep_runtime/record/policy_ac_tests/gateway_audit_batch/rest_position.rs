use super::*;

#[tokio::test]
async fn given_two_apis_with_empty_first_and_terminal_pages_when_position_cursors_resume_then_each_middle_resource_and_exact_parent_edge_survives()
-> TestResult {
    // Given pinned REST plans and two independent API owners with distinct native position tokens.
    let compiled = compile(
        "api-gateway",
        "2015-07-09",
        &["GetRestApis", "GetResources"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new("rest-empty-position-pages".to_owned())?;
    let root = catalog
        .lookup("api-gateway", "GetRestApis")
        .ok_or("REST root absent")?;
    let scope = ScopeKey::new(
        account.clone(),
        region.clone(),
        ServiceId::parse("api-gateway")?,
        OperationId::parse("GetRestApis")?,
        None,
    )?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rest-position.sqlite");
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
            input::root_bindings(root, Some(&account), Some(&region))?.into_bytes(),
        )?)
        .await?;
    let mut first_pages = 0_u8;
    let mut middle_pages = 0_u8;
    let mut terminal_pages = 0_u8;
    // When native requests, empty pages, continuations and terminal commits pass through the real wire/row/store consumers.
    for at in 0..7 {
        let lease = store
            .claim(&sweep, at, 10_000)
            .await?
            .ok_or("position continuation was lost")?;
        let plan = catalog
            .lookup("api-gateway", lease.scope().operation().as_str())
            .ok_or("REST lease plan absent")?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        assert_eq!(wire.method(), http::Method::GET);
        assert!(wire.body().is_empty());
        let mut phase = 0_u8;
        let body = if plan.operation_name() == "GetRestApis" {
            assert_eq!(wire.uri().path(), "/restapis");
            assert!(wire.uri().query().is_none());
            json!({"item":[{"id":"api-a"},{"id":"api-b"}]})
        } else {
            assert_eq!(plan.operation_name(), "GetResources");
            let api = text(&actual, &compiled.fields, "restApiId")?;
            assert!(["api-a", "api-b"].contains(&api.as_str()));
            assert_eq!(wire.uri().path(), format!("/restapis/{api}/resources"));
            let position = match record::path_value(
                actual.root(),
                &[compiled
                    .fields
                    .id("position")
                    .ok_or("position field absent")?],
            )? {
                None => None,
                Some(ValueRef::Text(value)) => Some(value),
                _ => return Err("position must be a string".into()),
            };
            if let Some(position) = position {
                assert_eq!(
                    wire.uri().query(),
                    Some(format!("position={position}").as_str())
                );
                if position == format!("middle-{api}") {
                    phase = 2;
                    middle_pages = middle_pages.checked_add(1).ok_or("middle count overflow")?;
                    json!({"item":[{"id":"shared","path":"/"}],"position":format!("terminal-{api}")})
                } else {
                    assert_eq!(
                        position,
                        format!("terminal-{api}"),
                        "cursor crossed API owners"
                    );
                    phase = 3;
                    terminal_pages = terminal_pages
                        .checked_add(1)
                        .ok_or("terminal count overflow")?;
                    json!({"item":[]})
                }
            } else {
                assert!(wire.uri().query().is_none());
                phase = 1;
                first_pages = first_pages.checked_add(1).ok_or("first count overflow")?;
                json!({"item":[],"position":format!("middle-{api}")})
            }
        };
        let response = decode(plan, &body)?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            lease.scope(),
            &actual,
            &response,
            &ancestry,
        )?;
        let unresolved = child::policy_dfgk_tests::projected_references(
            &catalog,
            &context,
            lease.scope(),
            &actual,
            &response,
            &observations,
        )?;
        if phase != 0 {
            assert!(children.is_empty());
            assert_eq!(observations.len(), if phase == 2 { 1 } else { 0 });
            assert_eq!(unresolved.len(), observations.len());
        }
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        assert_eq!(
            next.is_some(),
            matches!(phase, 1 | 2),
            "empty terminal page must finish, empty first page must continue"
        );
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
    // Then reopen retains exactly one parent-qualified middle-page resource per API, not phantom empty-page rows.
    assert_eq!((first_pages, middle_pages, terminal_pages), (2, 2, 2));
    assert!(store.claim(&sweep, 8, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 100).await?;
    let actual_ids = resources
        .into_iter()
        .map(|row| (row.key.kind.to_string(), row.key.id.to_string()))
        .collect::<BTreeSet<_>>();
    let expected_ids = [
        ("AWS::ApiGateway::RestApi".to_owned(), "api-a".to_owned()),
        ("AWS::ApiGateway::RestApi".to_owned(), "api-b".to_owned()),
        (
            "AWS::ApiGateway::Resource".to_owned(),
            "5:api-a|6:shared".to_owned(),
        ),
        (
            "AWS::ApiGateway::Resource".to_owned(),
            "5:api-b|6:shared".to_owned(),
        ),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(actual_ids, expected_ids);
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 2);
    let actual_edges = edges
        .edges
        .into_iter()
        .map(|edge| {
            assert_eq!(edge.source, "reviewed-rule");
            assert_eq!(edge.from.kind.as_str(), "AWS::ApiGateway::Resource");
            assert_eq!(edge.to.kind.as_str(), "AWS::ApiGateway::RestApi");
            assert_eq!(edge.relation.as_str(), "IN_REST_API");
            assert_eq!(edge.from.account, account);
            assert_eq!(edge.to.account, account);
            assert_eq!(edge.from.region, region);
            assert_eq!(edge.to.region, region);
            (edge.from.id.to_string(), edge.to.id.to_string())
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_edges,
        [
            ("5:api-a|6:shared".to_owned(), "api-a".to_owned()),
            ("5:api-b|6:shared".to_owned(), "api-b".to_owned())
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    );
    Ok(())
}
