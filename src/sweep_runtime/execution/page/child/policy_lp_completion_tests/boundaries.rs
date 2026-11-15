use super::*;

type RejectedCase = (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, Json, Option<(&'static str, Json)>, &'static str, &'static str, &'static str);

fn rejected_cases() -> Vec<RejectedCase> {
    vec![
        ("lex-models-v2", "2020-08-07", "ListBots", "ListBotAliases", "AWS::Lex::Bot", "AWS::Lex::BotAlias", json!({"botSummaries":[{"botId":"BOT0000001"}]}), None, "botAliasSummaries", "botAliasId", "ALIAS00001"),
        ("lex-models-v2", "2020-08-07", "ListBots", "ListBotVersions", "AWS::Lex::Bot", "AWS::Lex::BotVersion", json!({"botSummaries":[{"botId":"BOT0000001"}]}), None, "botVersionSummaries", "botVersion", "1"),
        ("neptune-graph", "2023-11-29", "ListGraphs", "ListPrivateGraphEndpoints", "AWS::NeptuneGraph::Graph", "AWS::NeptuneGraph::PrivateGraphEndpoint", json!({"graphs":[{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567890","id":"g-1234567890"}]}), None, "privateGraphEndpoints", "vpcId", "vpc-11111111"),
        ("pca-connector-ad", "2018-05-10", "ListConnectors", "ListTemplateGroupAccessControlEntries", "AWS::PCAConnectorAD::Connector", "AWS::PCAConnectorAD::TemplateGroupAccessControlEntry", json!({"Connectors":[{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:connector/11111111-1111-1111-1111-111111111111"}]}), Some(("ListTemplates", json!({"Templates":[{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:connector/11111111-1111-1111-1111-111111111111/template/22222222-2222-2222-2222-222222222222"}]}))), "AccessControlEntries", "GroupSecurityIdentifier", "S-1-5-21-1"),
    ]
}

#[tokio::test]
async fn given_a_persisted_child_page_when_a_mixed_native_continuation_is_rejected_then_prior_rows_references_edges_and_cursor_survive_reopen() -> TestResult {
    for (service, version, root, child, root_kind, child_kind, roots, middle, items, field, valid) in rejected_cases() {
        // Given real compiled plans and discovered native parents, not synthetic child work.
        let mut operations = vec![root];
        if let Some((operation, _)) = &middle { operations.push(operation); }
        operations.push(child);
        let bytes = compiled(service, version, &operations, root_kind)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let scenario = Scenario::new(&catalog, child).await?;
        scenario.seed(plan(&catalog, service, root)?).await?;
        let lease = scenario.claim().await?;
        let callable = plan(&catalog, service, root)?;
        let (request, _) = scenario.request(callable, &lease)?;
        scenario.apply(&catalog, callable, lease, &request, roots).await?;
        if let Some((operation, response)) = middle {
            let lease = scenario.claim().await?;
            assert_eq!(lease.scope().operation().as_str(), operation);
            let callable = plan(&catalog, service, operation)?;
            let (request, _) = scenario.request(callable, &lease)?;
            scenario.apply(&catalog, callable, lease, &request, response).await?;
        }
        let scenario = reject_continuation(scenario, &catalog, plan(&catalog, service, child)?, child_kind, items, field, valid).await?;
        assert_eq!(scenario.stored(root_kind).await?.0.len(), 1);
    }
    Ok(())
}

async fn reject_continuation<'a, 'catalog>(
    scenario: Scenario<'a, 'catalog>, catalog: &'a RuntimeCatalog<'catalog>, callable: &CallablePlan<'catalog>,
    child_kind: &str, items: &str, field: &str, valid: &str,
) -> TestResult<Scenario<'a, 'catalog>> {
    // Given a successfully committed child page and a nonempty durable continuation.
    let lease = scenario.claim().await?;
    assert_eq!(lease.scope().operation().as_str(), callable.operation_name());
    assert!(lease.token().is_none());
    let (request, wire) = scenario.request(callable, &lease)?;
    assert_parent(&wire, &bound(callable, &request)?)?;
    let token_field = if callable.service_name() == "pca-connector-ad" { "NextToken" } else { "nextToken" };
    let token = format!("resume-{}", callable.operation_name());
    scenario.apply(catalog, callable, lease, &request, json!({(items):[row(field, valid)],(token_field):token})).await?;
    let scenario = scenario.reopen().await?;
    let lease = scenario.claim().await?;
    let checkpoint = lease.token().ok_or("successful first page lost its continuation")?.to_vec();
    let before = boundary_snapshot(&scenario).await?;
    let saved = scenario.stored(child_kind).await?;
    assert_eq!(saved.0.len(), 1);
    assert_eq!(saved.1.len(), 1);
    let (request, wire) = scenario.request(callable, &lease)?;
    assert_boundary_cursor(&wire, token_field, &token)?;
    // When a different valid row precedes an empty identity on the continuation.
    let fresh = match field { "botAliasId" => "ALIAS00002", "botVersion" => "2", "vpcId" => "vpc-22222222",
        "GroupSecurityIdentifier" => "S-1-5-21-2", _ => return Err("unexpected rejected identity fixture".into()) };
    let response = json!({(items):[row(field, fresh),row(field, "")],(token_field):"must-not-commit"});
    let error = scenario.apply(catalog, callable, lease, &request, response).await.err().ok_or("empty correlated continuation was committed")?;
    assert!(error.to_string().contains("modeled identity text operand is empty"), "{error}");
    // Then every previous row, reference, edge and checkpoint byte survives reopening.
    let scenario = scenario.reopen().await?;
    assert_eq!(boundary_snapshot(&scenario).await?, before);
    assert_eq!(scenario.stored(child_kind).await?, saved);
    let retry = scenario.store.claim(&scenario.sweep, 20_000, 10_000).await?.ok_or("rejected continuation checkpoint was lost")?;
    assert_eq!(retry.scope().operation().as_str(), callable.operation_name());
    assert_eq!(retry.token(), Some(checkpoint.as_slice()));
    let (_, wire) = scenario.request(callable, &retry)?;
    assert_boundary_cursor(&wire, token_field, &token)?;
    Ok(scenario)
}

fn assert_boundary_cursor(wire: &http::Request<bytes::Bytes>, field: &str, expected: &str) -> TestResult {
    let query = wire.uri().query().and_then(|query| query.split('&').filter_map(|part| part.split_once('='))
        .find_map(|(key, value)| key.eq_ignore_ascii_case(field).then_some(value)));
    let body = if wire.body().is_empty() { None } else {
        serde_json::from_slice::<Json>(wire.body())?.get(field).and_then(Json::as_str).map(str::to_owned)
    };
    assert_eq!(query.or(body.as_deref()), Some(expected));
    Ok(())
}

async fn boundary_snapshot(scenario: &Scenario<'_, '_>) -> TestResult<Vec<Vec<String>>> {
    let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(scenario.directory.path().join("inventory.db")).read_only(true)).await?;
    let mut tables = Vec::new();
    for query in [
        "SELECT quote(sweep_id)||quote(scope_key)||quote(state)||quote(generation)||quote(lease_until)||quote(token)||quote(bindings)||quote(context) FROM work_queue ORDER BY sweep_id,scope_key",
        "SELECT quote(account)||quote(region)||quote(kind)||quote(id)||quote(name)||quote(content_hash)||quote(first_seen)||quote(last_changed) FROM resources ORDER BY account,region,kind,id",
        "SELECT quote(kind)||quote(id)||quote(record) FROM resource_facets ORDER BY kind,id,record",
        "SELECT quote(from_account)||quote(from_region)||quote(from_kind)||quote(from_id)||quote(target_scope)||quote(target_kind)||quote(target_path)||quote(target_value)||quote(relation)||quote(source)||quote(resolution) FROM unresolved_refs ORDER BY from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source",
        "SELECT quote(from_account)||quote(from_region)||quote(from_kind)||quote(from_id)||quote(to_account)||quote(to_region)||quote(to_kind)||quote(to_id)||quote(relation)||quote(source) FROM edges ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source",
    ] { tables.push(sqlx::query_scalar::<_, String>(query).fetch_all(&reader).await?); }
    reader.close().await;
    Ok(tables)
}

#[tokio::test]
async fn given_a_native_graph_with_empty_request_parent_field_when_child_work_is_attempted_then_no_phantom_endpoint_can_persist() -> TestResult {
    // The parent ARN is valid; its separate queryable id must not become an empty correlated operand.
    let bytes = compiled("neptune-graph", "2023-11-29", &["ListGraphs", "ListPrivateGraphEndpoints"], "AWS::NeptuneGraph::Graph")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "neptune-empty-request-parent").await?;
    let root = plan(&catalog, "neptune-graph", "ListGraphs")?;
    scenario.seed(root).await?;
    let lease = scenario.claim().await?;
    let (request, _) = scenario.request(root, &lease)?;
    let response = json!({"graphs":[{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567890","id":""}]});
    let failed_operation = if scenario.apply(&catalog, root, lease, &request, response).await.is_ok() {
        let child = plan(&catalog, "neptune-graph", "ListPrivateGraphEndpoints")?;
        let lease = scenario.claim().await?;
        assert_eq!(lease.scope().operation().as_str(), "ListPrivateGraphEndpoints");
        // Rejecting an empty HTTP label before projection is also fail-closed behavior.
        if let Ok((request, _)) = scenario.request(child, &lease) {
            let response = page(json!({"privateGraphEndpoints":[row("vpcId", "vpc-11111111")]}), "nextToken", false)?;
            let error = scenario.apply(&catalog, child, lease, &request, response).await.err().ok_or("empty correlated request parent was committed")?;
            assert!(error.to_string().contains("modeled identity text operand is empty"), "{error}");
        }
        "ListPrivateGraphEndpoints"
    } else { "ListGraphs" };
    let scenario = scenario.reopen().await?;
    assert_eq!(scenario.stored("AWS::NeptuneGraph::PrivateGraphEndpoint").await?, (BTreeSet::new(), BTreeSet::new()));
    let retry = scenario.store.claim(&scenario.sweep, 20_000, 10_000).await?.ok_or("empty-parent page checkpoint was lost")?;
    assert_eq!(retry.scope().operation().as_str(), failed_operation);
    assert!(retry.token().is_none());
    Ok(())
}

fn row(field: &str, value: &str) -> Json {
    if field == "vpcId" {
        json!({"vpcId":value,"subnetIds":["subnet-11111111"],"status":"AVAILABLE"})
    } else {
        json!({(field):value})
    }
}
