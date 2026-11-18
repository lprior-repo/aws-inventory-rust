use super::*;

mod catalog;
mod deadline;
mod descendants;

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
            "/catalog/field-ids.txt"
        )))
        .map_err(|error| error.to_string())
    });

// Expected identities deliberately do not call the identity compiler/projector under test.
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

fn page_size(service: &str, operation: &str) -> TestResult<usize> {
    Ok(match (service, operation) {
        (
            "evs",
            "ListEnvironments"
            | "ListEnvironmentHosts"
            | "ListEnvironmentConnectors"
            | "ListEnvironmentVlans",
        ) => 100,
        (
            "deadline",
            "ListFarms"
            | "ListFleets"
            | "ListQueues"
            | "ListLimits"
            | "ListBudgets"
            | "ListStorageProfiles"
            | "ListQueueEnvironments"
            | "ListQueueFleetAssociations"
            | "ListQueueLimitAssociations"
            | "ListVolumes",
        ) => 100,
        (
            "entityresolution",
            "ListIdMappingWorkflows"
            | "ListMatchingWorkflows"
            | "ListIdMappingJobs"
            | "ListMatchingJobs",
        ) => 25,
        (
            "fis",
            "ListExperimentTemplates" | "ListExperiments" | "ListTargetAccountConfigurations",
        ) => 100,
        ("frauddetector", "GetDetectors") => 10,
        ("datazone", "ListDomains") => 25,
        ("datazone", "ListProjectProfiles") => 50,
        ("secrets-manager", "ListSecrets") => 100,
        _ => return Err(format!("unknown page size for {service}.{operation}").into()),
    })
}
fn json_target(service: &str) -> TestResult<&'static str> {
    match service {
        "evs" => Ok("AmazonElasticVMwareService"),
        "frauddetector" => Ok("AWSHawksNestServiceFacade"),
        "secrets-manager" => Ok("secretsmanager"),
        _ => Err(format!("service {service} is not AWS JSON POST protocol").into()),
    }
}
fn is_rpc(service: &str) -> bool {
    service == "fms"
}
fn build_rpc_expected(
    service: &str,
    operation: &str,
    second: bool,
    parent_id: Option<&str>,
) -> TestResult<Json> {
    let mut fields = serde_json::Map::new();
    if let Some(field) = json_binding(service, operation) {
        let value = parent_id.ok_or(format!("missing parent for {service}.{operation}"))?;
        fields.insert(field.to_owned(), json!(value));
    }
    if second {
        fields.insert("NextToken".to_owned(), json!("same-next-token"));
    }
    Ok(json!(fields))
}
fn rpc_path(service: &str, operation: &str) -> TestResult<String> {
    match service {
        "fms" => Ok(format!("/service/AWSFMS_20180101/operation/{operation}")),
        _ => Err(format!("unknown RPC service {service}").into()),
    }
}
fn assert_rpc(
    wire: &http::Request<bytes::Bytes>,
    service: &str,
    operation: &str,
    expected: &Json,
) -> TestResult {
    assert_eq!(
        wire.method(),
        http::Method::POST,
        "expected POST for {service}.{operation}"
    );
    let expected_path = rpc_path(service, operation)?;
    assert_eq!(
        wire.uri().path(),
        &expected_path,
        "RPC path for {service}.{operation}"
    );
    assert!(
        wire.uri().query().is_none(),
        "RPC query must be None for {service}.{operation}"
    );
    assert!(
        wire.headers().get("x-amz-target").is_none(),
        "RPC should not have x-amz-target for {service}.{operation}"
    );
    let content_type = wire
        .headers()
        .get("content-type")
        .ok_or("RPC content-type missing")?
        .to_str()?;
    assert_eq!(
        content_type, "application/cbor",
        "RPC content-type for {service}.{operation}"
    );
    let protocol = wire
        .headers()
        .get("smithy-protocol")
        .ok_or("RPC smithy-protocol missing")?
        .to_str()?;
    assert_eq!(
        protocol, "rpc-v2-cbor",
        "RPC smithy-protocol for {service}.{operation}"
    );
    let body = wire.body();
    let expected_map = expected
        .as_object()
        .ok_or("expected RPC body is not an object")?;
    let mut decoder = minicbor::Decoder::new(body);
    let mut actual = serde_json::Map::new();
    if let Some(n) = decoder.map()? {
        let n = usize::try_from(n).map_err(|_| "RPC body map size exceeds usize")?;
        for _ in 0..n {
            let key = decoder.str()?.to_owned();
            if actual.contains_key(&key) {
                return Err(
                    format!("duplicate field {key} in RPC body for {service}.{operation}").into(),
                );
            }
            let value = decode_cbor_value(&mut decoder)?;
            actual.insert(key, value);
        }
    } else {
        loop {
            if decoder.datatype()? == minicbor::data::Type::Break {
                decoder.skip()?;
                break;
            }
            let key = decoder.str()?.to_owned();
            if actual.contains_key(&key) {
                return Err(
                    format!("duplicate field {key} in RPC body for {service}.{operation}").into(),
                );
            }
            let value = decode_cbor_value(&mut decoder)?;
            actual.insert(key, value);
        }
    }
    assert_eq!(
        actual.len(),
        expected_map.len(),
        "RPC body field count mismatch for {service}.{operation}"
    );
    assert_eq!(
        decoder.position(),
        body.len(),
        "trailing CBOR data in RPC body for {service}.{operation}"
    );
    assert_eq!(expected_map, &actual, "RPC body for {service}.{operation}");
    Ok(())
}
fn decode_cbor_value(decoder: &mut minicbor::Decoder<'_>) -> TestResult<Json> {
    match decoder.datatype()? {
        minicbor::data::Type::U8
        | minicbor::data::Type::U16
        | minicbor::data::Type::U32
        | minicbor::data::Type::U64 => Ok(json!(decoder.u64()?)),
        minicbor::data::Type::String => Ok(json!(decoder.str()?)),
        _ => Err("unsupported CBOR type in RPC body".into()),
    }
}
fn json_binding(service: &str, operation: &str) -> Option<&'static str> {
    match (service, operation) {
        ("fms", "ListResourceSetResources") => Some("Identifier"),
        ("frauddetector", "DescribeDetector") => Some("detectorId"),
        _ => None,
    }
}
pub fn json_to_cbor(value: &Json) -> TestResult<Vec<u8>> {
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encode_json(&mut encoder, value)?;
    Ok(encoder.into_writer())
}
fn encode_json(encoder: &mut minicbor::Encoder<Vec<u8>>, value: &Json) -> TestResult {
    match value {
        Json::Null => {
            encoder
                .null()
                .map_err(|e| format!("CBOR null error: {e}"))?;
        }
        Json::Bool(b) => {
            encoder
                .bool(*b)
                .map_err(|e| format!("CBOR bool error: {e}"))?;
        }
        Json::Number(n) => {
            if let Some(i) = n.as_i64() {
                encoder.i64(i).map_err(|e| format!("CBOR i64 error: {e}"))?;
            } else if let Some(u) = n.as_u64() {
                encoder.u64(u).map_err(|e| format!("CBOR u64 error: {e}"))?;
            } else {
                return Err("unsupported JSON number".into());
            }
        }
        Json::String(s) => {
            encoder.str(s).map_err(|e| format!("CBOR str error: {e}"))?;
        }
        Json::Array(arr) => {
            encoder
                .array(u64::try_from(arr.len()).map_err(|_| "array length exceeds u64")?)
                .map_err(|e| format!("CBOR array error: {e}"))?;
            for item in arr {
                encode_json(encoder, item)?;
            }
        }
        Json::Object(map) => {
            encoder
                .map(u64::try_from(map.len()).map_err(|_| "map length exceeds u64")?)
                .map_err(|e| format!("CBOR map error: {e}"))?;
            for (key, val) in map {
                encoder
                    .str(key)
                    .map_err(|e| format!("CBOR key error: {e}"))?;
                encode_json(encoder, val)?;
            }
        }
    }
    Ok(())
}
fn detector_id_from_arn(arn: &str) -> TestResult<&str> {
    arn.split_once('/')
        .map(|(_, tail)| tail)
        .ok_or_else(|| "detector ARN has no resource path".into())
}
fn evs_environment_id(parent_arn: &str) -> TestResult<&str> {
    parent_arn
        .split_once('/')
        .map(|(_, tail)| tail)
        .ok_or_else(|| "environment ARN has no resource path".into())
}
fn json_token_field(service: &str) -> &'static str {
    if matches!(service, "fms" | "secrets-manager") {
        "NextToken"
    } else {
        "nextToken"
    }
}
fn json_size_field(service: &str) -> &'static str {
    if matches!(service, "fms" | "secrets-manager") {
        "MaxResults"
    } else {
        "maxResults"
    }
}
fn assert_get(
    wire: &http::Request<bytes::Bytes>,
    path: &str,
    service: &str,
    operation: &str,
    second: bool,
) -> TestResult {
    assert_eq!(
        wire.method(),
        http::Method::GET,
        "expected GET for {service}.{operation}"
    );
    assert_eq!(
        wire.uri().path(),
        path,
        "GET path for {service}.{operation}"
    );
    assert!(
        wire.body().is_empty(),
        "GET body must be empty for {service}.{operation}"
    );
    assert!(
        wire.headers().get("x-amz-target").is_none(),
        "GET should not have x-amz-target for {service}.{operation}"
    );
    let query = wire.uri().query().unwrap_or_default();
    if (service, operation) == ("fis", "ListExperimentTargetAccountConfigurations") {
        let expected = if second {
            "nextToken=same-next-token"
        } else {
            ""
        };
        assert_eq!(query, expected, "GET query for {service}.{operation}");
    } else {
        let size = page_size(service, operation)?;
        let expected = if second {
            format!("maxResults={size}&nextToken=same-next-token")
        } else {
            format!("maxResults={size}")
        };
        assert_eq!(query, expected, "GET query for {service}.{operation}");
    }
    Ok(())
}
fn assert_post(
    wire: &http::Request<bytes::Bytes>,
    service: &str,
    operation: &str,
    second: bool,
    parent_id: Option<&str>,
) -> TestResult {
    assert_eq!(
        wire.method(),
        http::Method::POST,
        "expected POST for {service}.{operation}"
    );
    assert_eq!(
        wire.uri().path(),
        "/",
        "POST path must be / for {service}.{operation}"
    );
    assert!(
        wire.uri().query().is_none(),
        "POST query must be None for {service}.{operation}"
    );
    let target_prefix = json_target(service)?;
    let header = wire
        .headers()
        .get("x-amz-target")
        .ok_or("AWS JSON target missing")?
        .to_str()?;
    let expected_target = format!("{}.{}", target_prefix, operation);
    assert_eq!(
        header, &expected_target,
        "AWS JSON target for {service}.{operation}"
    );
    let encoded: Json = serde_json::from_slice(wire.body())?;
    let mut expected = serde_json::Map::new();
    if let Some(field) = json_binding(service, operation) {
        let value = parent_id.ok_or(format!("missing parent for {service}.{operation}"))?;
        if service == "frauddetector" {
            let id = detector_id_from_arn(value)?;
            expected.insert(field.to_owned(), json!(id));
        } else {
            expected.insert(field.to_owned(), json!(value));
        }
    }
    if service == "evs"
        && operation.starts_with("ListEnvironment")
        && operation != "ListEnvironments"
    {
        let value = parent_id.ok_or("missing parent for EVS child operation")?;
        let env_id = evs_environment_id(value)?;
        expected.insert("environmentId".to_owned(), json!(env_id));
    }
    if service == "secrets-manager" {
        expected.insert("IncludePlannedDeletion".to_owned(), json!(true));
    }
    if operation != "GetNotificationChannel" && operation != "DescribeDetector" {
        let size = page_size(service, operation)?;
        expected.insert(json_size_field(service).to_owned(), json!(size));
    }
    if second {
        expected.insert(
            json_token_field(service).to_owned(),
            json!("same-next-token"),
        );
    }
    assert_eq!(
        encoded,
        Json::Object(expected),
        "POST body for {service}.{operation}"
    );
    Ok(())
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
