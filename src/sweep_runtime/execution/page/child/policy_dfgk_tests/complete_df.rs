use super::*;

mod catalog;
mod deadline;
mod descendants;

type StoredEdge = (String, String, String, String, String);
type StoredResourceKey = (String,String,String,String);
type ScopedStoredEdge = (String,String,String,String,String,String,String,String,String);
type Nodes = BTreeSet<(String, String)>;
type Edges = BTreeSet<StoredEdge>;
type Response = fn(&str, &Record, bool) -> TestResult<(Json, Option<String>)>;

static FIELDS: std::sync::LazyLock<Result<catalog_tool::smithy::Registry, String>> =
    std::sync::LazyLock::new(|| catalog_tool::smithy::Registry::parse(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt"))
    ).map_err(|error| error.to_string()));

// Expected identities deliberately do not call the identity compiler/projector under test.
fn key(parts: &[&str]) -> String {
    parts.iter().map(|part| format!("{}:{part}", part.len())).collect::<Vec<_>>().join("|")
}
fn string<'a>(request: &'a Record, name: &str) -> TestResult<&'a str> {
    let field = FIELDS.as_ref().map_err(|error| error.clone())?.id(name)
        .ok_or_else(|| format!("pinned request field absent: {name}"))?;
    match record::path_value(request.root(), &[field])? {
        Some(ValueRef::Text(value)) => Ok(value),
        _ => Err(format!("bound request omitted {name}").into()),
    }
}
fn paginated(mut body: Json, second: bool, token: &str) -> TestResult<Json> {
    if !second {
        body.as_object_mut().ok_or("response must be an object")?.insert(token.to_owned(), json!("same-next-token"));
    }
    Ok(body)
}
fn leaf(second: bool) -> &'static str { if second { "second" } else { "first" } }
fn node(nodes: &mut Nodes, kind: &str, id: &str) { nodes.insert((kind.to_owned(), id.to_owned())); }
fn edge(edges: &mut Edges, kind: &str, id: &str, parent_kind: &str, parent: &str, relation: &str) {
    edges.insert((kind.to_owned(), id.to_owned(), parent_kind.to_owned(), parent.to_owned(), relation.to_owned()));
}

async fn run(service: &str, roots: &[&str], expected_requests: usize, response: Response, nodes: Nodes, edges: Edges) -> TestResult {
    run_with_dependencies(service, roots, &[], expected_requests, response, nodes, edges).await
}

async fn run_with_dependencies(service: &str, roots: &[&str], dependencies: &[(&str, &str)], expected_requests: usize, response: Response, nodes: Nodes, edges: Edges) -> TestResult {
    let catalog = catalog::catalog()?;
    let scenario = Scenario::new(&catalog, &format!("complete-df-{service}")).await?;
    for operation in roots { scenario.seed(fixture::plan(&catalog, service, operation)?).await?; }
    for (dependency,operation) in dependencies { scenario.seed(fixture::plan(&catalog,dependency,operation)?).await?; }
    let mut requests = BTreeSet::new();
    for _ in 0..expected_requests {
        let lease = scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.ok_or("production child or continuation was lost")?;
        let operation = lease.scope().operation().as_str().to_owned();
        let call_service=lease.scope().service().as_str();
        let plan = fixture::plan(&catalog, call_service, &operation)?;
        let (request, wire) = scenario.prepared_request(plan, &lease)?;
        let second = lease.token().is_some();
        assert!(requests.insert((operation.clone(), lease.scope().parent().map(|parent| parent.id.as_str().to_owned()), second)), "duplicate child/page request");
        let (body, path) = response(&operation, &request, second)?;
        if let Some(path) = path {
            assert_eq!(wire.method(), http::Method::GET);
            assert_eq!(wire.uri().path(), path);
            let query = wire.uri().query().unwrap_or_default();
            // Page sizes derived from pinned Smithy model range constraints (max of range).
            // evs: ListEnvironments maxResults range 1-100 -> 100
            // deadline: ListFarms maxResults range 1-100 -> 100
            // entityresolution: ListIdMappingWorkflows/ListMatchingWorkflows maxResults range max=25 -> 25
            // fis: ListExperimentTemplates/ListExperiments maxResults range 1-100 -> 100
            // frauddetector: GetDetectors maxResults range 5-10 -> 10
            // datazone: ListDomains maxResults range 1-25 -> 25
            let page_size=match (call_service,operation.as_str()) {
                ("evs","ListEnvironments")=>100,
                ("deadline","ListFarms")=>100,
                ("entityresolution","ListIdMappingWorkflows"|"ListMatchingWorkflows")=>25,
                ("fis","ListExperimentTemplates"|"ListExperiments")=>100,
                ("frauddetector","GetDetectors")=>10,
                ("datazone","ListDomains")=>25,
                _=>return Err(format!("unknown page size for {call_service}.{operation}").into()),
            };
            let expected_query=if second {
                format!("maxResults={page_size}&nextToken=same-next-token")
            } else {
                format!("maxResults={page_size}")
            };
            assert_eq!(query, expected_query, "unreviewed query filter on {call_service}.{operation}");
        } else {
            assert_eq!(wire.method(), http::Method::POST);
            assert!(wire.headers().get("x-amz-target").ok_or("AWS JSON target missing")?.to_str()?.ends_with(&format!(".{operation}")));
            let encoded: Json = serde_json::from_slice(wire.body())?;
            let mut expected=serde_json::Map::new();
            let binding=match (call_service,operation.as_str()) {
                ("evs","ListEnvironmentHosts"|"ListEnvironmentConnectors"|"ListEnvironmentVlans")=>Some("environmentId"),
                ("fms","ListResourceSetResources")=>Some("Identifier"),
                ("frauddetector","DescribeDetector")=>Some("detectorId"),
                _=>None,
            };
            if let Some(field)=binding {expected.insert(field.to_owned(),json!(string(&request,field)?));}
            if call_service=="secrets-manager" {expected.insert("IncludePlannedDeletion".to_owned(),json!(true));}
            // Page sizes derived from pinned Smithy model range constraints (max of range).
            // evs: ListEnvironments maxResults range 1-100 -> 100
            // deadline: ListFarms maxResults range 1-100 -> 100
            // entityresolution: ListIdMappingWorkflows/ListMatchingWorkflows maxResults range max=25 -> 25
            // fis: ListExperimentTemplates/ListExperiments maxResults range 1-100 -> 100
            // frauddetector: GetDetectors maxResults range 5-10 -> 10
            // datazone: ListDomains maxResults range 1-25 -> 25
            let page_size=match (call_service,operation.as_str()) {
                ("evs","ListEnvironments")=>100,
                ("deadline","ListFarms")=>100,
                ("entityresolution","ListIdMappingWorkflows"|"ListMatchingWorkflows")=>25,
                ("fis","ListExperimentTemplates"|"ListExperiments")=>100,
                ("frauddetector","GetDetectors")=>10,
                ("datazone","ListDomains")=>25,
                _=>return Err(format!("unknown page size for {call_service}.{operation}").into()),
            };
            expected.insert("maxResults",json!(page_size));
            if second {
                let token=if matches!(call_service,"fms"|"secrets-manager") {"NextToken"} else {"nextToken"};
                expected.insert(token.to_owned(),json!("same-next-token"));
            }
            assert_eq!(encoded,Json::Object(expected),"unreviewed native request filter on {call_service}.{operation}");
        }
        scenario.apply(&catalog, plan, lease, &request, body).await?;
    }
    assert!(scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.is_none(), "unexpected/unreviewed work remained");
    let path = scenario.directory.path().join("inventory.db");
    let Scenario { directory, store, sweep, .. } = scenario;
    drop(store);
    let reopened = BinaryStore::open(&path).await?;
    assert!(reopened.claim(&sweep, 2000, 10_000).await?.is_none());
    let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path).read_only(true)).await?;
    let actual_nodes: Vec<StoredResourceKey> = sqlx::query_as("SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id").fetch_all(&reader).await?;
    let expected_nodes: Vec<_>=nodes.into_iter().map(|(kind,id)|("123456789012".to_owned(),"us-east-1".to_owned(),kind,id)).collect();
    assert_eq!(actual_nodes,expected_nodes);
    let actual_owners:Vec<StoredResourceKey>=sqlx::query_as("SELECT account,region,kind,id FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(&reader).await?;
    assert_eq!(actual_owners,expected_nodes,"wrong or duplicate durable resource ownership after reopen");
    let actual_edges: Vec<ScopedStoredEdge> = sqlx::query_as("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation FROM edges ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source").fetch_all(&reader).await?;
    let expected_edges:Vec<_>=edges.into_iter().map(|(kind,id,parent_kind,parent,relation)|("123456789012".to_owned(),"us-east-1".to_owned(),kind,id,"123456789012".to_owned(),"us-east-1".to_owned(),parent_kind,parent,relation)).collect();
    assert_eq!(actual_edges,expected_edges);
    reader.close().await;
    drop(reopened);
    drop(directory);
    Ok(())
}
