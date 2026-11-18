use super::*;

pub(super) async fn filtered_roots(
    service: &str,
    date: &str,
    cases: &[(&str, &str, &str, &str, &str, &str)],
) -> TestResult {
    // Given genuine enum-bearing pinned operations, not hand-authored callable plans.
    let excluded: &[&str] = match service {
        "auditmanager" => &[
            "GetAssessmentFramework",
            "GetControl",
            "GetEvidence",
            "CreateAssessment",
        ],
        "bedrock-data-automation" => &[
            "GetBlueprint",
            "GetDataAutomationProject",
            "InvokeBlueprintOptimizationAsync",
        ],
        _ => return Err("unreviewed owned-enum fixture family".into()),
    };
    let operations = cases
        .iter()
        .map(|case| case.0)
        .chain(excluded.iter().copied())
        .collect::<Vec<_>>();
    let compiled = compile(service, date, &operations)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for operation in excluded {
        assert!(
            catalog.lookup(service, operation).is_none(),
            "content or write plan became callable: {operation}"
        );
    }
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("owned-enums.sqlite");
    let mut store = BinaryStore::create(
        &path,
        SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
    )
    .await?;
    let sweep = SweepId::new(format!("{service}-owned-enums"))?;
    for (operation, _, _, _, _, _) in cases {
        let plan = catalog
            .lookup(service, operation)
            .ok_or("filtered root absent")?;
        let scope = ScopeKey::new(
            account.clone(),
            region.clone(),
            ServiceId::parse(service)?,
            OperationId::parse(operation)?,
            None,
        )?;
        let input = input::root_bindings(plan, Some(&account), Some(&region))?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                input.into_bytes(),
            )?)
            .await?;
    }
    let mut expected_keys = BTreeSet::new();
    // When native wire requests, opaque continuation tokens and canonical rows traverse SQLite reopen boundaries.
    for at in 0..=8 {
        let Some(lease) = store.claim(&sweep, at, 10_000).await? else {
            break;
        };
        let plan = catalog
            .lookup(service, lease.scope().operation().as_str())
            .ok_or("filtered lease absent")?;
        let (_, items, id, field, value, kind) = cases
            .iter()
            .find(|case| case.0 == plan.operation_name())
            .ok_or("unreviewed operation scheduled")?;
        assert_eq!(plan.kind().as_str(), *kind);
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        if service == "auditmanager" {
            assert!(
                wire.uri()
                    .query()
                    .ok_or("custom enum query absent")?
                    .split('&')
                    .any(|pair| pair == format!("{field}={value}"))
            );
        } else {
            assert_eq!(
                serde_json::from_slice::<Value>(wire.body())?
                    .get(*field)
                    .and_then(Value::as_str),
                Some(*value)
            );
        }
        assert_eq!(text(&actual, &compiled.fields, field)?, *value);
        let resumed = lease.token().is_some();
        if resumed {
            assert_eq!(text(&actual, &compiled.fields, "nextToken")?, "later");
        }
        let id_suffix = if resumed {
            "12345678-1234-1234-1234-123456789002"
        } else {
            "12345678-1234-1234-1234-123456789001"
        };
        let resource = match plan.operation_name() {
            "ListAssessmentFrameworks" => "assessmentFramework",
            "ListControls" => "control",
            "ListBlueprints" => "blueprint",
            "ListDataAutomationProjects" => "data-automation-project",
            _ => return Err("unexpected owned resource ARN family".into()),
        };
        let arn = format!(
            "arn:aws:{}:us-east-1:123456789012:{resource}/{id_suffix}",
            if service == "auditmanager" {
                "auditmanager"
            } else {
                "bedrock"
            }
        );
        let mut body = json!({*items:[{*id:arn}]});
        if !resumed {
            body["nextToken"] = json!("later");
        }
        let response = decode(plan, &body)?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            lease.scope(),
            &actual,
            &response,
            &DiscoveryContext::decode(lease.context())?,
        )?;
        assert!(children.is_empty());
        assert_eq!(observations.len(), 1);
        let expected = resource_key(&account, &region, kind, &arn)?;
        assert_eq!(
            observations.first().ok_or("owned row absent")?.key(),
            &expected
        );
        assert!(
            expected_keys.insert(expected),
            "duplicate native owned-root fixture key"
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
                unresolved: Vec::new(),
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
    // Then exact canonical kinds, account, region and native IDs survive the independent resource read model.
    assert!(store.claim(&sweep, 9, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(
        resources.len(),
        cases
            .len()
            .checked_mul(2)
            .ok_or("owned page count overflow")?
    );
    assert_eq!(resources.len(), expected_keys.len());
    assert_eq!(
        resources
            .into_iter()
            .map(|resource| resource.key)
            .collect::<BTreeSet<_>>(),
        expected_keys
    );
    Ok(())
}
