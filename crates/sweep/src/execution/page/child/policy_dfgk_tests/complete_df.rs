use super::*;

mod catalog;
mod deadline;
mod descendants;
mod http_checks;
mod wire;
use http_checks::{assert_get, assert_post};
pub use wire::json_to_cbor;
use wire::{assert_rpc, build_rpc_expected, is_rpc, json_binding, json_target, page_size};

type StoredEdge = (String, String, String, String, String);
type StoredResourceKey = (String, String, String, String);
type ScopedStoredEdge = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);
type Nodes = BTreeSet<(String, String)>;
type Edges = BTreeSet<StoredEdge>;
type Response = fn(&str, &Record, bool) -> TestResult<(Json, Option<String>)>;

static FIELDS: std::sync::LazyLock<Result<catalog_tool::smithy::Registry, String>> =
    std::sync::LazyLock::new(|| {
        catalog_tool::smithy::Registry::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../catalog/field-ids.txt"
        )))
        .map_err(|error| error.to_string())
    });

fn key(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| format!("{}:{part}", part.len()))
        .collect::<Vec<_>>()
        .join("|")
}
fn string<'a>(request: &'a Record, name: &str) -> TestResult<&'a str> {
    let field = FIELDS
        .as_ref()
        .map_err(|error| error.clone())?
        .id(name)
        .ok_or_else(|| format!("pinned request field absent: {name}"))?;
    match record::path_value(request.root(), &[field])? {
        Some(ValueRef::Text(value)) => Ok(value),
        _ => Err(format!("bound request omitted {name}").into()),
    }
}
fn paginated(mut body: Json, second: bool, token: &str) -> TestResult<Json> {
    if !second {
        body.as_object_mut()
            .ok_or("response must be an object")?
            .insert(token.to_owned(), json!("same-next-token"));
    }
    Ok(body)
}
fn leaf(second: bool) -> &'static str {
    if second { "second" } else { "first" }
}
fn node(nodes: &mut Nodes, kind: &str, id: &str) {
    nodes.insert((kind.to_owned(), id.to_owned()));
}
fn edge(edges: &mut Edges, kind: &str, id: &str, parent_kind: &str, parent: &str, relation: &str) {
    edges.insert((
        kind.to_owned(),
        id.to_owned(),
        parent_kind.to_owned(),
        parent.to_owned(),
        relation.to_owned(),
    ));
}

async fn run(
    service: &str,
    roots: &[&str],
    expected_requests: usize,
    response: Response,
    nodes: Nodes,
    edges: Edges,
) -> TestResult {
    run_with_dependencies(
        service,
        roots,
        &[],
        expected_requests,
        response,
        nodes,
        edges,
    )
    .await
}

async fn run_with_dependencies(
    service: &str,
    roots: &[&str],
    dependencies: &[(&str, &str)],
    expected_requests: usize,
    response: Response,
    nodes: Nodes,
    edges: Edges,
) -> TestResult {
    let catalog = catalog::catalog()?;
    let scenario = Scenario::new(&catalog, &format!("complete-df-{service}")).await?;
    for operation in roots {
        scenario
            .seed(fixture::plan(&catalog, service, operation)?)
            .await?;
    }
    for (dependency, operation) in dependencies {
        scenario
            .seed(fixture::plan(&catalog, dependency, operation)?)
            .await?;
    }
    let mut requests = BTreeSet::new();
    for _ in 0..expected_requests {
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10_000)
            .await?
            .ok_or("production child or continuation was lost")?;
        let operation = lease.scope().operation().as_str().to_owned();
        let call_service = lease.scope().service().as_str();
        let plan = fixture::plan(&catalog, call_service, &operation)?;
        let (request, wire) = scenario.prepared_request(plan, &lease)?;
        let second = lease.token().is_some();
        let parent_id = lease.scope().parent().map(|parent| parent.id.as_str());
        assert!(
            requests.insert((
                operation.clone(),
                lease
                    .scope()
                    .parent()
                    .map(|parent| parent.id.as_str().to_owned()),
                second
            )),
            "duplicate child/page request"
        );
        let (body, path) = response(&operation, &request, second)?;
        if let Some(path) = path {
            assert_get(&wire, &path, call_service, &operation, second)?;
        } else if is_rpc(call_service) {
            let expected_data = build_rpc_expected(call_service, &operation, second, parent_id)?;
            assert_rpc(&wire, call_service, &operation, &expected_data)?;
        } else {
            assert_post(&wire, call_service, &operation, second, parent_id)?;
        }
        scenario
            .apply(&catalog, plan, lease, &request, body)
            .await?;
    }
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 1001, 10_000)
            .await?
            .is_none(),
        "unexpected/unreviewed work remained"
    );
    let path = scenario.directory.path().join("inventory.db");
    let Scenario {
        directory,
        store,
        sweep,
        ..
    } = scenario;
    drop(store);
    let reopened = BinaryStore::open(&path).await?;
    assert!(reopened.claim(&sweep, 2000, 10_000).await?.is_none());
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .read_only(true),
        )
        .await?;
    let actual_nodes: Vec<StoredResourceKey> = sqlx::query_as(
        "SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id",
    )
    .fetch_all(&reader)
    .await?;
    let expected_nodes: Vec<_> = nodes
        .into_iter()
        .map(|(kind, id)| ("123456789012".to_owned(), "us-east-1".to_owned(), kind, id))
        .collect();
    assert_eq!(actual_nodes, expected_nodes);
    let actual_owners: Vec<StoredResourceKey> = sqlx::query_as("SELECT account,region,kind,id FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(&reader).await?;
    assert_eq!(
        actual_owners, expected_nodes,
        "wrong or duplicate durable resource ownership after reopen"
    );
    let actual_edges: Vec<ScopedStoredEdge> = sqlx::query_as("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation FROM edges ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source").fetch_all(&reader).await?;
    let expected_edges: Vec<_> = edges
        .into_iter()
        .map(|(kind, id, parent_kind, parent, relation)| {
            (
                "123456789012".to_owned(),
                "us-east-1".to_owned(),
                kind,
                id,
                "123456789012".to_owned(),
                "us-east-1".to_owned(),
                parent_kind,
                parent,
                relation,
            )
        })
        .collect();
    assert_eq!(actual_edges, expected_edges);
    reader.close().await;
    drop(reopened);
    drop(directory);
    Ok(())
}
