use super::*;
#[tokio::test]
async fn given_unattached_safe_roots_when_empty_pages_resume_after_sqlite_reopen_then_exact_resources_and_connector_edges_survive_without_forbidden_calls()
-> TestResult {
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let cases = cases::all();
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("gk-roots.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&path, registry).await?;
    for case in &cases {
        let sweep = SweepId::new(format!("roots-{}-{}", case.service, case.operation))?;
        seed(
            &store,
            &catalog,
            std::slice::from_ref(case),
            &sweep,
            &account,
            &home,
        )
        .await?;
        for page in 0..if case.token.is_some() { 3 } else { 1 } {
            assert!(
                consume(
                    &store,
                    &catalog,
                    &consumers,
                    case,
                    (&sweep, &account, &home),
                    page,
                )
                .await?
            );
            drop(store);
            store = BinaryStore::open(&path).await?;
        }
        assert!(
            store.claim(&sweep, 4, 10_000).await?.is_none(),
            "{} scheduled an extra call",
            case.operation
        );
    }
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(resources.len(), 36);
    let actual = resources
        .iter()
        .map(|resource| {
            (
                resource.key.kind.as_str().to_owned(),
                resource.key.id.as_str().to_owned(),
            )
        })
        .collect::<BTreeSet<_>>();
    let expected = cases
        .iter()
        .flat_map(|case| case.ids.iter().map(|id| (case.kind.to_owned(), id.clone())))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 2);
    for edge in edges.edges {
        assert_eq!(edge.from.kind.as_str(), "AWS::InspectorV2::Connector");
        assert_eq!(
            edge.to.kind.as_str(),
            "AWS::ApiOnly::Inspector2ConnectorScanConfiguration"
        );
        assert_eq!(edge.relation.as_str(), "USES_SCAN_CONFIGURATION");
        let ordinal = if edge.from.id.as_str().ends_with("0001") {
            "first"
        } else {
            "later"
        };
        assert!(edge.to.id.as_str().ends_with(ordinal));
    }
    Ok(())
}

#[test]
fn given_safe_metadata_promotions_when_compiling_embedded_plans_then_content_secrets_paid_mutations_and_provider_reference_calls_stay_absent()
-> TestResult {
    let catalog = fixture::catalog()?;
    let approved = cases::all()
        .into_iter()
        .map(|case| (case.service, case.operation))
        .collect::<BTreeSet<_>>();
    for (service, _, operations) in fixture::SERVICES {
        for operation in *operations {
            if !approved.contains(&(*service, *operation)) {
                assert!(
                    catalog.lookup(service, operation).is_none(),
                    "forbidden {service}.{operation} became callable"
                );
            }
        }
    }
    assert!(
        catalog
            .lookup("imagebuilder", "ListImageBuildVersions")
            .is_none(),
        "failed component-step output is content, even when only ARNs are projected"
    );
    Ok(())
}

#[test]
fn given_trusted_root_scope_when_an_arn_is_malformed_or_foreign_then_no_observation_or_child_is_admitted()
-> TestResult {
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let sweep = SweepId::new("gk-root-arn-boundary".to_owned())?;
    for case in cases::all() {
        let identity = match (case.service, case.operation) {
            ("gamelift", "ListScripts") => "ScriptArn",
            ("gamelift", "ListLocations") => "LocationArn",
            ("imagebuilder", _) => "arn",
            ("inspector2", "ListConnectors") => "connectorArn",
            ("inspector2", "ListConnectorScanConfigurations") => "awsConfigConnectorArn",
            ("invoicing", _) => "ProcurementPortalPreferenceArn",
            _ => continue,
        };
        let plan = catalog
            .lookup(case.service, case.operation)
            .ok_or("ARN root plan absent")?;
        let region = if plan.scope() == PlanScope::Account {
            Region::Global
        } else {
            Region::Regional(home.clone())
        };
        let scope = ScopeKey::new(
            account.clone(),
            region.clone(),
            ServiceId::parse(case.service)?,
            OperationId::parse(case.operation)?,
            None,
        )?;
        let request = input::root_bindings(plan, Some(&account), Some(&region))?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let row = case.rows.first().ok_or("ARN fixture row absent")?;
        let valid = cbor::response(plan, &json!({case.items:[row]}))?;
        let valid = record::policy_dfgk_tests::fixture::decode(plan, &valid)?;
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            &scope,
            &request,
            &valid,
            &DiscoveryContext::empty(),
        )?;
        assert_eq!(observations.len(), 1);
        assert!(children.is_empty());
        let mut parts = case
            .ids
            .first()
            .ok_or("ARN fixture identity absent")?
            .splitn(6, ':');
        let (
            Some("arn"),
            Some(partition),
            Some(service),
            Some(arn_region),
            Some(arn_account),
            Some(resource),
        ) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        )
        else {
            return Err("fixture identity is not an ARN".into());
        };
        for forged in [
            "not-an-arn".to_owned(),
            format!("arn:{partition}:{service}:{arn_region}:999999999999:{resource}"),
            format!("arn:aws-cn:{service}:{arn_region}:{arn_account}:{resource}"),
            format!("arn:{partition}:{service}:us-west-2:{arn_account}:{resource}"),
        ] {
            let mut forged_row = row.clone();
            forged_row
                .as_object_mut()
                .ok_or("row is not an object")?
                .insert(identity.to_owned(), json!(forged));
            let response = cbor::response(plan, &json!({case.items:[forged_row]}))?;
            let response = record::policy_dfgk_tests::fixture::decode(plan, &response)?;
            let error = child::policy_dfgk_tests::project_children(
                &catalog,
                &context,
                &scope,
                &request,
                &response,
                &DiscoveryContext::empty(),
            )
            .err()
            .ok_or("foreign ARN was admitted")?;
            assert!(
                error.to_string().contains("modeled ARN"),
                "{}: {error}",
                case.operation
            );
        }
    }
    Ok(())
}
