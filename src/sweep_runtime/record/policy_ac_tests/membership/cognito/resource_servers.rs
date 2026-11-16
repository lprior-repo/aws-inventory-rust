use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { ProviderOnlyFirst, NullOwner, NonStringOwner, UnrelatedPool }

async fn exercise(mode: Mode) -> TestResult {
    // Given two pools, actual pinned ResourceServer shapes, and their embedded ownership/correlation policy.
    let compiled = compile("cognito-identity-provider", "2016-04-18", &["ListUserPools", "ListResourceServers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new("cognito-resource-server-ownership".to_owned())?;
    let root = catalog.lookup("cognito-identity-provider", "ListUserPools").ok_or("pool plan absent")?;
    let scope = ScopeKey::new(account.clone(), region.clone(), ServiceId::parse("cognito-identity-provider")?, OperationId::parse("ListUserPools")?, None)?;
    let bindings = input::root_bindings(root, Some(&account), Some(&region))?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("resource-servers.sqlite");
    let mut store = BinaryStore::create(&path, SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?).await?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, bindings.into_bytes())?).await?;
    let mut filtered_pages = 0_u8;
    let mut owned_pages = 0_u8;
    let mut rejected = false;
    // When native wire pages traverse the production row predicate before projection and atomic SQLite commits.
    for at in 0..5 {
        let lease = store.claim(&sweep, at, 10_000).await?.ok_or("expected native inventory page absent")?;
        let plan = catalog.lookup("cognito-identity-provider", lease.scope().operation().as_str()).ok_or("leased plan absent")?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        let is_child = plan.operation_name() == "ListResourceServers";
        let later = lease.token().is_some();
        let pool = if is_child { text(&actual, &compiled.fields, "UserPoolId")? } else { String::new() };
        let body = if !is_child {
            NativeRequest::post("/".to_owned(), json!({"MaxResults":60})).assert_matches(&wire)?;
            pools()
        } else {
            assert!(!pool.is_empty());
            assert_eq!(lease.scope().parent().ok_or("resource server work lost pool")?.id.as_str(), pool);
            let native = if later { json!({"UserPoolId":pool,"NextToken":"later"}) } else { json!({"UserPoolId":pool}) };
            NativeRequest::post("/".to_owned(), native).assert_matches(&wire)?;
            let provider = json!({"UserPoolId":pool,"Identifier":"provider","Name":"Cognito managed","ManagedBy":"COGNITO"});
            let owned = json!({"UserPoolId":pool,"Identifier":"same","Name":"owned","Scopes":[{"ScopeName":"read","ScopeDescription":"metadata"}]});
            if mode == Mode::ProviderOnlyFirst {
                if later { json!({"ResourceServers":[owned,provider]}) }
                else { json!({"ResourceServers":[provider],"NextToken":"later"}) }
            } else {
                let bad = match mode {
                    Mode::NullOwner => json!({"UserPoolId":pool,"Identifier":"bad","ManagedBy":null}),
                    Mode::NonStringOwner => json!({"UserPoolId":pool,"Identifier":"bad","ManagedBy":false}),
                    Mode::UnrelatedPool => json!({"UserPoolId":"us-east-1_Unrelated","Identifier":"bad"}),
                    Mode::ProviderOnlyFirst => return Err("provider-only mode reached invalid-row branch".into()),
                };
                json!({"ResourceServers":[owned,bad]})
            }
        };
        let result = decode(plan, &body).and_then(|response| -> TestResult<_> {
            let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
            let ancestry = DiscoveryContext::decode(lease.context())?;
            let (observations, children) = child::policy_dfgk_tests::project_children(&catalog, &context, lease.scope(), &actual, &response, &ancestry)?;
            let actual_branch = crate::sweep_runtime::execution::routing::execution_branch(plan, lease.scope().branch())?;
            let mut selected = Vec::new();
            for row in record::item_records(&response, plan.items())? {
                if crate::sweep_runtime::row_policy::selected(plan, actual_branch, &row, &actual, &response)? { selected.push(row); }
            }
            let unresolved = rows::references(&catalog, plan, &home, &selected, &observations, &actual, None)?;
            let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
            Ok((observations, children, unresolved, next))
        });
        if is_child && mode != Mode::ProviderOnlyFirst {
            assert!(result.is_err(), "malformed ownership or cross-pool selected key must reject the atomic page");
            rejected = true;
            break;
        }
        let (observations, children, unresolved, next) = result?;
        if is_child {
            assert!(children.is_empty(), "provider references must not schedule descendants");
            assert_eq!(observations.len(), if later { 1 } else { 0 });
            assert_eq!(unresolved.len(), observations.len());
            assert_eq!(next.is_some(), !later, "an all-provider page must retain its continuation token");
            if later {
                owned_pages = owned_pages.checked_add(1).ok_or("owned page count overflow")?;
                let observation = observations.first().ok_or("owned second-page resource absent")?;
                assert_eq!(observation.key().id.as_str(), format!("{}:{pool}|4:same", pool.len()));
                assert_eq!(unresolved.first().ok_or("pool join absent")?.target.value().as_str(), pool);
            } else { filtered_pages = filtered_pages.checked_add(1).ok_or("filtered page count overflow")?; }
        }
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| (Some(next.cursor.into_bytes()), next.continuation_digest));
        store.commit_page(PageCommit { lease, observations, children, unresolved, edges: Vec::new(), tags: Vec::new(), next_token, continuation_digest, done, observed_at_ms: at }).await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then only customer-created canonical rows can persist; a bad second row cannot partially commit its good predecessor.
    drop(store);
    let store = BinaryStore::open(&path).await?;
    let resources = store.projection_resources(None, 100).await?;
    let servers = resources.iter().filter(|resource| resource.key.kind.as_str() == "AWS::Cognito::UserPoolResourceServer").collect::<Vec<_>>();
    if mode == Mode::ProviderOnlyFirst {
        assert!(!rejected);
        assert_eq!((filtered_pages, owned_pages), (2, 2));
        assert_eq!(servers.len(), 2);
        let actual_ids = servers.iter().map(|resource| resource.key.id.to_string()).collect::<BTreeSet<_>>();
        let expected_ids = POOLS.map(|pool| format!("{}:{pool}|4:same", pool.len())).into_iter().collect::<BTreeSet<_>>();
        assert_eq!(actual_ids, expected_ids);
        let edges = store.projection_edges(None, 100).await?;
        assert_eq!(edges.edges.len(), 2);
        for edge in edges.edges {
            assert_eq!(edge.relation.as_str(), "IN_USER_POOL");
            assert_eq!(edge.from.kind.as_str(), "AWS::Cognito::UserPoolResourceServer");
            assert_eq!(edge.to.kind.as_str(), "AWS::Cognito::UserPool");
            let pool = edge.to.id.as_str();
            assert!(POOLS.contains(&pool));
            assert_eq!(edge.from.id.as_str(), format!("{}:{pool}|4:same", pool.len()));
            assert_eq!(edge.from.account, account);
            assert_eq!(edge.to.account, account);
            assert_eq!(edge.from.region, region);
            assert_eq!(edge.to.region, region);
        }
        assert!(store.claim(&sweep, 6, 10_000).await?.is_none());
    } else {
        assert!(rejected);
        assert!(servers.is_empty());
        assert_eq!(resources.len(), 2, "only the previously committed pools may remain");
        assert!(store.projection_edges(None, 100).await?.edges.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn given_provider_only_first_pages_in_two_pools_when_cursors_resume_then_only_owned_servers_and_exact_pool_edges_are_committed() -> TestResult {
    exercise(Mode::ProviderOnlyFirst).await
}

#[tokio::test]
async fn given_a_good_row_followed_by_null_non_string_or_unrelated_pool_ownership_when_a_page_is_consumed_then_no_partial_resource_or_edge_is_committed() -> TestResult {
    for mode in [Mode::NullOwner, Mode::NonStringOwner, Mode::UnrelatedPool] { exercise(mode).await?; }
    Ok(())
}
