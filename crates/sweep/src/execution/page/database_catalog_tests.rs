use super::*;
use crate::execution::test_transport::{Resources, TestResult};
use catalog_tool::{classes::DenyList, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, OperationId, Region, RegionId, ScopeKey, ServiceId},
};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec};
use sqlx::Row;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::SocketAddr,
    path::Path,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const CALLS: [(&str, &str); 6] = [
    ("rds", "DescribeDBClusters"),
    ("rds", "DescribeDBInstances"),
    ("docdb", "DescribeDBClusters"),
    ("docdb", "DescribeDBInstances"),
    ("neptune", "DescribeDBClusters"),
    ("neptune", "DescribeDBInstances"),
];
const SECRET_MARKER: &str = "fixture-rds-secret-marker-7f3a";

#[tokio::test]
async fn given_database_catalogs_when_swept_then_rows_are_owned_and_excluded_values_are_not_persisted()
-> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let bytes = compile_catalog(address)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let namespaces = namespaces(&catalog)?;
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
    let sweep = SweepId::new("rds-family-row-policy-consumer".to_owned())?;
    seed_calls(&catalog, &store, &sweep).await?;
    let resources = Resources::new("us-east-1")?;
    let progress = crate::control::Progress::new(None);
    let context = resources.runtime(&catalog, &store, &progress)?;
    let peer = tokio::spawn(serve_query(listener, namespaces));
    for _ in CALLS {
        let lease = store
            .claim(&sweep, super::now_ms()?, 60_000)
            .await?
            .ok_or("database call lease absent")?;
        let processed = super::process(&context, lease, false).await?;
        assert_eq!(processed.summary.rows, 1);
    }
    let seen = peer
        .await
        .map_err(|error| std::io::Error::other(format!("loopback peer task failed: {error}")))?
        .map_err(std::io::Error::other)?;
    assert_eq!(seen, CALLS.into_iter().map(|(s, o)| (s.to_owned(), o.to_owned())).collect());
    assert_persisted_rows(directory.path()).await?;
    Ok(())
}

fn compile_catalog(address: SocketAddr) -> TestResult<Vec<u8>> {
    const SERVICES: [(&str, &str, &[&str]); 3] = [
        ("rds", "2014-10-31", &["DescribeDBClusters", "DescribeDBInstances"]),
        ("docdb", "2014-10-31", &["DescribeDBClusters", "DescribeDBInstances"]),
        ("neptune", "2014-10-31", &["DescribeDBClusters", "DescribeDBInstances"]),
    ];
    let source = tempfile::tempdir()?;
    let models = source.path().join("models");
    let embedded = overlay::embedded()?;
    let mut policies = Vec::new();
    let mut endpoint_map = BTreeMap::new();
    for (service, version, operations) in SERVICES {
        crate::pinned_model::install(&models, service, version, operations)?;
        let source_policy = embedded
            .iter()
            .find(|policy| policy.service() == service)
            .ok_or("database service policy absent")?;
        let selected = operations
            .iter()
            .map(|operation| {
                let policy = source_policy
                    .operations()
                    .iter()
                    .find(|(name, _)| name.as_str() == *operation)
                    .map(|(_, policy)| policy.clone())
                    .ok_or("database operation policy absent")?;
                Ok((*operation, policy))
            })
            .collect::<TestResult<Vec<_>>>()?;
        policies.push(overlay::ServiceOverlay::new(service, selected)?);
        endpoint_map.insert(
            service.to_owned(),
            crate::execution::test_catalog::loopback_endpoint(service, address)?,
        );
    }
    let checked = overlay::check_policies(source.path(), &policies, &DenyList::builtin())?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fields = smithy::Registry::parse(&fs::read_to_string(root.join("catalog/field-ids.txt"))?)?;
    let base = smithy::build_catalog(source.path(), &fields, "rds-family-consumer")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoint_map)?)
}

fn namespaces(catalog: &RuntimeCatalog<'_>) -> TestResult<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for (service, operation) in CALLS {
        let plan = catalog.lookup(service, operation).ok_or("database plan absent")?;
        let namespace = plan
            .service()
            .wire()?
            .xml_namespace()?
            .ok_or("Query XML namespace absent")?;
        result.insert(service.to_owned(), namespace.uri.to_owned());
    }
    Ok(result)
}

async fn seed_calls(
    catalog: &RuntimeCatalog<'_>,
    store: &BinaryStore,
    sweep: &SweepId,
) -> TestResult {
    let account = AccountId::parse("123456789012")?;
    let region_id = RegionId::parse("us-east-1")?;
    for (service, operation) in CALLS {
        let plan = catalog.lookup(service, operation).ok_or("database plan absent")?;
        let scope = ScopeKey::new(
            account.clone(),
            Region::Regional(region_id.clone()),
            ServiceId::parse(service)?,
            OperationId::parse(operation)?,
            None,
        )?;
        let bindings = input::root_bindings(plan, Some(&account), Some(scope.region()))?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
    }
    Ok(())
}

async fn serve_query(
    listener: TcpListener,
    namespaces: BTreeMap<String, String>,
) -> Result<BTreeSet<(String, String)>, String> {
    let mut seen = BTreeSet::new();
    for _ in CALLS {
        let (mut socket, _) = listener.accept().await.map_err(|error| error.to_string())?;
        let (headers, body) = read_request(&mut socket).await?;
        let service = request_service(&headers)?.to_owned();
        let operation = request_action(&body)?.to_owned();
        if service == "docdb" && operation == "DescribeDBInstances"
            && body.split('&').any(|pair| pair.starts_with("Filters.") || pair.starts_with("Filter."))
        {
            return Err("DocumentDB instance discovery sent an unsupported filter".into());
        }
        seen.insert((service.clone(), operation.clone()));
        let namespace = namespaces.get(&service).ok_or("unknown Query service")?;
        let response = query_response(&service, &operation, namespace);
        let wire = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
            response.len()
        );
        socket
            .write_all(wire.as_bytes())
            .await
            .map_err(|error| error.to_string())?;
        socket.shutdown().await.map_err(|error| error.to_string())?;
    }
    Ok(seen)
}

async fn read_request(socket: &mut TcpStream) -> Result<(String, String), String> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 2048];
    for _ in 0..16 {
        let count = socket.read(&mut buffer).await.map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("request ended before body".to_owned());
        }
        bytes.extend_from_slice(buffer.get(..count).ok_or("request buffer overflow")?);
        let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
            continue;
        };
        let end = end.checked_add(4).ok_or("header offset overflow")?;
        let (header_bytes, body_bytes) = bytes
            .split_at_checked(end)
            .ok_or("request header exceeds buffer")?;
        let headers = std::str::from_utf8(header_bytes).map_err(|error| error.to_string())?;
        let length = request_header(headers, "content-length")?
            .parse::<usize>()
            .map_err(|error| error.to_string())?;
        if body_bytes.len() == length {
            let body = std::str::from_utf8(body_bytes).map_err(|error| error.to_string())?;
            return Ok((headers.to_owned(), body.to_owned()));
        }
    }
    Err("request exceeds loopback read budget".to_owned())
}

fn request_service(headers: &str) -> Result<&str, String> {
    let authorization = request_header(headers, "authorization")?;
    let scope = authorization
        .split("Credential=")
        .nth(1)
        .and_then(|value| value.split(',').next())
        .ok_or("SigV4 credential absent")?;
    scope
        .split('/')
        .nth(3)
        .ok_or_else(|| "SigV4 service absent".to_owned())
}

fn request_action(body: &str) -> Result<&str, String> {
    body.split('&')
        .find_map(|pair| pair.strip_prefix("Action="))
        .ok_or_else(|| "Query action absent".to_owned())
}

fn request_header<'a>(headers: &'a str, name: &str) -> Result<&'a str, String> {
    headers
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.trim())
        })
        .ok_or_else(|| format!("{name} header absent"))
}

fn query_response(service: &str, operation: &str, namespace: &str) -> String {
    let (collection, items) = match operation {
        "DescribeDBClusters" => ("DBClusters", cluster_items(service)),
        "DescribeDBInstances" => ("DBInstances", instance_items(service)),
        _ => ("Unknown", String::new()),
    };
    format!(
        "<{operation}Response xmlns=\"{namespace}\"><{operation}Result><{collection}>{items}</{collection}></{operation}Result></{operation}Response>",
    )
}

fn cluster_items(service: &str) -> String {
    let rows = match service {
        "rds" => vec![
            ("rds-cluster-1", "arn:aws:rds:us-east-1:123456789012:cluster:rds-1", "postgres"),
            ("docdb-cluster-1", "arn:aws:rds:us-east-1:123456789012:cluster:docdb-1", "docdb"),
            ("neptune-cluster-1", "arn:aws:rds:us-east-1:123456789012:cluster:neptune-1", "neptune"),
        ],
        "docdb" => vec![("docdb-cluster-1", "arn:aws:rds:us-east-1:123456789012:cluster:docdb-1", "docdb")],
        "neptune" => vec![("neptune-cluster-1", "arn:aws:rds:us-east-1:123456789012:cluster:neptune-1", "neptune")],
        _ => Vec::new(),
    };
    rows.into_iter()
        .map(|(id, arn, engine)| {
            format!(
                "<DBCluster><DBClusterIdentifier>{id}</DBClusterIdentifier><DbClusterResourceId>{id}</DbClusterResourceId><DBClusterArn>{arn}</DBClusterArn><Engine>{engine}</Engine>{}</DBCluster>",
                row_supplement()
            )
        })
        .collect()
}

fn instance_items(service: &str) -> String {
    let rows = match service {
        "rds" | "docdb" => vec![
            ("dbi-rds-1", "rds-1", "postgres"),
            ("dbi-docdb-1", "docdb-1", "docdb"),
            ("dbi-neptune-1", "neptune-1", "neptune"),
        ],
        "neptune" => vec![("dbi-neptune-1", "neptune-1", "neptune")],
        _ => Vec::new(),
    };
    rows.into_iter()
        .map(|(id, name, engine)| {
            format!(
                "<DBInstance><DBInstanceIdentifier>{name}</DBInstanceIdentifier><DbiResourceId>{id}</DbiResourceId><DBInstanceArn>arn:aws:rds:us-east-1:123456789012:db:{name}</DBInstanceArn><Engine>{engine}</Engine>{}</DBInstance>",
                row_supplement()
            )
        })
        .collect()
}

fn row_supplement() -> String {
    format!(
        "<PendingModifiedValues><MasterUserPassword>{SECRET_MARKER}</MasterUserPassword></PendingModifiedValues><TagList><Tag><Key>fixture</Key><Value>synthetic</Value></Tag></TagList>"
    )
}

async fn assert_persisted_rows(directory: &Path) -> TestResult {
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.join("inventory.db"))
                .read_only(true),
        )
        .await?;
    let rows = sqlx::query("SELECT kind,id,name FROM resources ORDER BY kind,id")
        .fetch_all(&reader)
        .await?;
    let actual = rows
        .iter()
        .map(|row| {
            let kind: String = row.try_get("kind")?;
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            assert!(!name.as_deref().is_some_and(|value| contains_marker(value.as_bytes())));
            Ok((kind, id))
        })
        .collect::<Result<BTreeSet<_>, sqlx::Error>>()?;
    let expected = BTreeSet::from([
        ("AWS::RDS::DBCluster".to_owned(), "rds-cluster-1".to_owned()),
        ("AWS::RDS::DBInstance".to_owned(), "dbi-rds-1".to_owned()),
        (
            "AWS::DocDB::DBCluster".to_owned(),
            "arn:aws:rds:us-east-1:123456789012:cluster:docdb-1".to_owned(),
        ),
        ("AWS::ApiOnly::DocDBDBInstance".to_owned(), "dbi-docdb-1".to_owned()),
        (
            "AWS::Neptune::DBCluster".to_owned(),
            "arn:aws:rds:us-east-1:123456789012:cluster:neptune-1".to_owned(),
        ),
        ("AWS::Neptune::DBInstance".to_owned(), "dbi-neptune-1".to_owned()),
    ]);
    assert_eq!(actual, expected);
    assert_marker_absent_from_facets(&reader).await?;
    assert_marker_absent_from_tags(&reader).await?;
    assert_marker_absent_from_edges(&reader).await?;
    reader.close().await;
    Ok(())
}

async fn assert_marker_absent_from_facets(reader: &sqlx::SqlitePool) -> TestResult {
    let rows = sqlx::query("SELECT record,owned,exclusions,relations,name FROM resource_facets")
        .fetch_all(reader)
        .await?;
    assert_eq!(rows.len(), 6);
    for row in rows {
        for column in ["record", "owned", "exclusions", "relations"] {
            let value: Vec<u8> = row.try_get(column)?;
            assert!(!contains_marker(&value));
        }
        let name: Option<String> = row.try_get("name")?;
        assert!(!name.as_deref().is_some_and(|value| contains_marker(value.as_bytes())));
    }
    Ok(())
}

async fn assert_marker_absent_from_tags(reader: &sqlx::SqlitePool) -> TestResult {
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT kind,id,tag_key,tag_value FROM tags ORDER BY kind,id,tag_key",
    )
        .fetch_all(reader)
        .await?;
    // Of the six pinned response shapes, only the two RDS shapes include TagList.
    let actual: Vec<_> = rows
        .iter()
        .map(|(kind, id, key, value)| (kind.as_str(), id.as_str(), key.as_str(), value.as_str()))
        .collect();
    assert_eq!(actual, [
        ("AWS::RDS::DBCluster", "rds-cluster-1", "fixture", "synthetic"),
        ("AWS::RDS::DBInstance", "dbi-rds-1", "fixture", "synthetic"),
    ]);
    Ok(())
}

async fn assert_marker_absent_from_edges(reader: &sqlx::SqlitePool) -> TestResult {
    let rows = sqlx::query("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges")
        .fetch_all(reader)
        .await?;
    for row in rows {
        for column in [
            "from_account", "from_region", "from_kind", "from_id", "to_account", "to_region",
            "to_kind", "to_id", "relation", "source",
        ] {
            let value: String = row.try_get(column)?;
            assert!(!contains_marker(value.as_bytes()));
        }
    }
    Ok(())
}

fn contains_marker(value: &[u8]) -> bool {
    value
        .windows(SECRET_MARKER.len())
        .any(|window| window == SECRET_MARKER.as_bytes())
}
