use super::*;
mod pages;
pub(super) async fn run(
    model: (&str, &str),
    root: &str,
    root_owner_field: &str,
    root_body: Json,
    operations: &[&str],
    excluded: &[&str],
    cases: Vec<Case>,
) -> TestResult {
    let (service, date) = model;
    // Given the actual pinned model, embedded policy, mappings, and two independently selected parents.
    let compiled = if service == "cleanrooms" {
        let mut operations = operations.to_vec();
        operations.push("ListConfiguredTables");
        crate::record::policy_ac_tests::fixture::compile_many(&[
            (service, date, &operations),
            (
                "cleanroomsml",
                "2023-09-06",
                &["ListConfiguredAudienceModels"],
            ),
            ("entityresolution", "2018-05-10", &["ListIdNamespaces"]),
        ])?
    } else {
        compile(service, date, operations)?
    };
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for operation in excluded {
        assert!(
            catalog.lookup(service, operation).is_none(),
            "excluded operation became callable: {operation}"
        );
    }
    for case in &cases {
        assert_eq!(
            catalog
                .lookup(service, case.operation)
                .ok_or("child plan absent")?
                .kind()
                .as_str(),
            case.kind
        );
    }
    let consumers = ConsumerIndex::build(&catalog)?;
    let parent = catalog.lookup(service, root).ok_or("parent plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new(format!("{service}-owned-metadata"))?;
    let scope = ScopeKey::new(
        account.clone(),
        region.clone(),
        ServiceId::parse(service)?,
        OperationId::parse(root)?,
        None,
    )?;
    let root_request = input::root_bindings(parent, Some(&account), Some(&region))?;
    let root_response = decode(parent, &root_body)?;
    let context = child::Context::new(parent, &sweep, &account, &home, &consumers);
    let (parents, children) = child::policy_dfgk_tests::project_children(
        &catalog,
        &context,
        &scope,
        &root_request,
        &root_response,
        &DiscoveryContext::empty(),
    )?;
    assert_eq!(parents.len(), 2);
    assert_eq!(
        children.len(),
        cases.len().checked_mul(2).ok_or("child count overflow")?
    );
    let (items, parent_kind, identity) = match service {
        "cleanrooms" => ("membershipSummaries", "AWS::CleanRooms::Membership", "arn"),
        "cognito-identity-provider" => ("UserPools", "AWS::Cognito::UserPool", "Id"),
        "connect" => ("InstanceSummaryList", "AWS::Connect::Instance", "Arn"),
        "connectcases" => ("domains", "AWS::Cases::Domain", "domainArn"),
        "connecthealth" => ("domains", "AWS::ApiOnly::ConnectHealthDomain", "arn"),
        _ => return Err("unreviewed membership root identity".into()),
    };
    let owners = root_body
        .get(items)
        .and_then(Json::as_array)
        .ok_or("native parent rows absent")?
        .iter()
        .map(|row| {
            let owner = row
                .get(root_owner_field)
                .and_then(Json::as_str)
                .ok_or("native owner selector absent")?;
            let id = row
                .get(identity)
                .and_then(Json::as_str)
                .ok_or("native parent identity absent")?;
            Ok((
                owner.to_owned(),
                resource_key(&account, &region, parent_kind, id)?,
            ))
        })
        .collect::<TestResult<BTreeMap<String, ResourceKey>>>()?;
    assert_eq!(owners.len(), 2);
    assert_eq!(
        parents
            .iter()
            .map(|row| row.key().clone())
            .collect::<BTreeSet<_>>(),
        owners.values().cloned().collect::<BTreeSet<_>>()
    );
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("owned-metadata.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&path, registry).await?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            root_request.as_bytes().to_vec(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, 0, 10_000)
        .await?
        .ok_or("parent lease absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations: parents,
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
    let expected = cases
        .iter()
        .flat_map(|case| {
            case.owners.iter().flat_map(move |owner| {
                owner
                    .pages
                    .iter()
                    .map(move |page| (case.kind.to_owned(), page.id.clone()))
            })
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected.len(),
        cases.len().checked_mul(4).ok_or("page count overflow")?
    );
    let mut expected_resources = owners.values().cloned().collect::<BTreeSet<_>>();
    for (kind, id) in &expected {
        expected_resources.insert(resource_key(&account, &region, kind, id)?);
    }
    let parent_relation = match service {
        "cleanrooms" => "IN_MEMBERSHIP",
        "cognito-identity-provider" => "IN_USER_POOL",
        "connect" => "IN_INSTANCE",
        "connectcases" | "connecthealth" => "IN_DOMAIN",
        _ => return Err("unreviewed membership parent relation".into()),
    };
    let mut expected_edges = BTreeSet::new();
    for case in &cases {
        for owner in &case.owners {
            let target = owners
                .get(&owner.value)
                .ok_or("fixture owner was not selected")?;
            for page in &owner.pages {
                expected_edges.insert((
                    case.kind.to_owned(),
                    page.id.clone(),
                    target.kind.to_string(),
                    target.id.to_string(),
                    parent_relation.to_owned(),
                ));
            }
        }
    }
    let (returned, seen) = pages::PageContext {
        service,
        catalog: &catalog,
        consumers: &consumers,
        registry: &compiled.fields,
        parent,
        account: &account,
        home: &home,
        region: &region,
        root_request: &root_request,
        owners: &owners,
        parent_relation,
        cases: &cases,
        path: &path,
        sweep: &sweep,
    }.consume(store, &expected).await?;
    store = returned;
    // Then every exact canonical/API-only row survives SQLite reopen, and no forbidden work remains.
    assert_eq!(seen, expected);
    assert!(store.claim(&sweep, 1000, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 1000).await?;
    assert_eq!(resources.len(), expected_resources.len());
    let persisted = resources
        .into_iter()
        .map(|resource| resource.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(persisted, expected_resources);
    let edges = store.projection_edges(None, 1000).await?;
    assert_eq!(edges.edges.len(), expected_edges.len());
    let persisted_edges = edges
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
    assert_eq!(persisted_edges, expected_edges);
    drop(store);
    directory.close()?;
    Ok(())
}
