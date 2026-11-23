use super::super::super::*;
use inventory_core::{
    catalog::runtime::{RelationTargetMode, RuntimeCatalog},
    inventory::{
        AccountId, Edge as StoredEdge, EdgeSource, Kind, Observation, OperationId, ReferenceTarget,
        Region, Relation, ResourceId, ResourceKey, ScopeKey, ServiceId, TagObservation,
        UnresolvedReference,
    },
    record::{Exclusions, FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use std::{collections::BTreeMap, error::Error, fmt::Write, path::PathBuf, time::SystemTime};

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(crate) const KIND: &str = "AWS::Example::Thing";
pub(crate) const ACCOUNT: &str = "111111111111";
pub(crate) const REGION: &str = "us-east-1";

pub(crate) fn node(account: &str, region: &str, kind: &str, id: &str) -> GraphNode {
    GraphNode {
        account: account.into(),
        region: region.into(),
        kind: kind.into(),
        id: id.into(),
    }
}

pub(crate) fn root() -> GraphNode {
    node(ACCOUNT, REGION, KIND, "shared")
}

pub(crate) fn key(node: &GraphNode) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    })
}

pub(crate) fn hex(bytes: &[u8]) -> TestResult<String> {
    let mut value = String::new();
    bytes
        .iter()
        .try_for_each(|byte| write!(value, "{byte:02x}"))?;
    Ok(value)
}

pub(crate) fn digest(node: &GraphNode) -> TestResult<String> {
    hex(key(node)?.digest().as_bytes())
}

pub(crate) fn arn(node: &GraphNode) -> String {
    format!(
        "arn:aws:example:{}:{}:thing/{}",
        node.region, node.account, node.id
    )
}

fn foreign(edge: &cmdb_view::GraphEdge, account: &str, region: &str) -> bool {
    edge.to.account != account || edge.to.region != region
}

pub(crate) fn edge(
    from: &GraphNode,
    to: &GraphNode,
    relation: &str,
    provenance: &str,
) -> cmdb_view::GraphEdge {
    cmdb_view::GraphEdge {
        from: from.clone(),
        to: to.clone(),
        relation: relation.into(),
        provenance: provenance.into(),
    }
}

pub(crate) fn ui_edge(edge: &cmdb_view::GraphEdge) -> TestResult<crate::data::Edge> {
    Ok(crate::data::Edge {
        from: digest(&edge.from)?,
        to: digest(&edge.to)?,
        label: edge.relation.clone(),
        provenance: edge.provenance.clone(),
    })
}

pub(crate) struct Fixture {
    _directory: tempfile::TempDir,
    pub path: PathBuf,
    pub store: BinaryStore,
}

impl Fixture {
    pub async fn new() -> TestResult<Self> {
        let directory = super::super::isolation::temporary_directory()?;
        let path = directory.path().join("inventory.sqlite");
        let bytes = super::catalog::catalog()?;
        let runtime = RuntimeCatalog::from_bytes(&bytes)?;
        let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
        let store = BinaryStore::create(&path, registry).await?;
        Ok(Self {
            _directory: directory,
            path,
            store,
        })
    }

    pub async fn commit(
        &self,
        sweep: &str,
        account: &str,
        region: &str,
        rows: &[(&str, &str)],
        links: &[cmdb_view::GraphEdge],
        at: i64,
    ) -> TestResult<Vec<Row>> {
        self.commit_tagged(sweep, (account, region), rows, links, vec![], at)
            .await
    }

    pub async fn commit_tagged(
        &self,
        sweep: &str,
        (account, region): (&str, &str),
        rows: &[(&str, &str)],
        links: &[cmdb_view::GraphEdge],
        tags: Vec<TagObservation>,
        at: i64,
    ) -> TestResult<Vec<Row>> {
        let scope = ScopeKey::new(
            AccountId::parse(account)?,
            Region::parse(region)?,
            ServiceId::parse("example")?,
            OperationId::parse("ListThings")?,
            None,
        )?;
        let sweep = SweepId::new(sweep.into())?;
        self.store
            .seed_work(WorkSpec::new(sweep.clone(), scope, None, vec![1])?)
            .await?;
        let lease = self
            .store
            .claim(&sweep, at, 100)
            .await?
            .ok_or("fixture lease missing")?;
        let observations = rows
            .iter()
            .map(|(id, name)| {
                let targets = links
                    .iter()
                    .filter(|edge| edge.from.id == *id)
                    .filter_map(|edge| {
                        if foreign(edge, account, region) {
                            Some(Value::Text(arn(&edge.to)))
                        } else {
                            rows.iter()
                                .find(|(target, _)| *target == edge.to.id)
                                .map(|(_, target)| Value::Text((*target).into()))
                        }
                    })
                    .collect::<Vec<_>>();
                let mut fields =
                    BTreeMap::from([(Key::Field(FieldId::new(1)?), Value::Text((*name).into()))]);
                if !targets.is_empty() {
                    fields.insert(Key::Field(FieldId::new(2)?), Value::List(targets));
                }
                let record = Record::encode(&Value::Map(fields))?;
                Ok(Observation::new(
                    key(&node(account, region, KIND, id))?,
                    record,
                    &Exclusions::none(),
                    Some(Arc::from(*name)),
                )?)
            })
            .collect::<TestResult<Vec<_>>>()?;
        // Independent oracle: original writer observations, NOT reader/projection output.
        let expected = observations
            .iter()
            .map(|value| {
                let key = value.key();
                Ok(Row {
                    account: key.account.as_str().into(),
                    region: key.region.as_str().into(),
                    kind: key.kind.as_str().into(),
                    id: key.id.as_str().into(),
                    name: value
                        .name()
                        .ok_or("fixture observation requires name")?
                        .into(),
                    key: hex(key.digest().as_bytes())?,
                    record_bytes: Some(value.record().as_bytes().len()),
                    content_hash: Some(hex(value.content_hash().as_bytes())?),
                    tags: tags
                        .iter()
                        .filter(|snapshot| snapshot.resource() == key)
                        .flat_map(|snapshot| snapshot.tags())
                        .map(|tag| cmdb_view::ResourceTag {
                            key: tag.key().into(),
                            value: tag.value().into(),
                        })
                        .collect(),
                    tags_truncated: false,
                    tag_observation: tags.iter().find(|snapshot| snapshot.resource() == key).map(
                        |snapshot| cmdb_view::StoredTagObservation {
                            coverage: snapshot.coverage(),
                            observed_at_ms: at,
                            last_complete_at_ms: (snapshot.coverage()
                                == inventory_core::inventory::TagCoverage::Complete)
                                .then_some(at),
                        },
                    ),
                })
            })
            .collect::<TestResult<Vec<_>>>()?;
        let (foreign_links, local_links): (Vec<_>, Vec<_>) = links
            .iter()
            .partition(|edge| foreign(edge, account, region));
        let edges = local_links
            .into_iter()
            .map(|edge| {
                Ok(StoredEdge {
                    from: key(&edge.from)?,
                    to: key(&edge.to)?,
                    relation: Relation::parse(&edge.relation)?,
                    source: EdgeSource::Model,
                })
            })
            .collect::<TestResult<Vec<_>>>()?;
        let unresolved = foreign_links
            .into_iter()
            .map(|edge| {
                Ok(UnresolvedReference {
                    from: key(&edge.from)?,
                    relation: Relation::parse(&edge.relation)?,
                    source: EdgeSource::InferredExact,
                    target: ReferenceTarget::typed(
                        AccountId::parse(&edge.to.account)?,
                        Region::parse(&edge.to.region)?,
                        Kind::parse(&edge.to.kind)?,
                        vec![FieldId::new(1)?],
                        ResourceId::parse(&arn(&edge.to))?,
                        RelationTargetMode::SelectedFields,
                    )?,
                })
            })
            .collect::<TestResult<Vec<_>>>()?;
        self.store
            .commit_page(PageCommit {
                lease,
                observations,
                edges,
                unresolved,
                tags,
                next_token: None,
                continuation_digest: None,
                children: vec![],
                done: true,
                observed_at_ms: at,
            })
            .await?;
        Ok(expected)
    }

    pub fn worker(&self, graph: Option<&NativeGraph>) -> DataWorker {
        DataWorker::start(
            WorkerConfig {
                database: Some(self.path.clone()),
                graph_url: graph.map(|value| value.endpoint.clone()),
                graph_name: graph.map(|value| value.name.clone()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
            SignalToUI::default(),
        )
    }
}

pub(crate) struct NativeGraph {
    pub endpoint: String,
    pub name: String,
    client: redis::Client,
    deleted: std::cell::Cell<bool>,
}

const REQUIRE_FALKOR: &str = "AWS_INVENTORY_REQUIRE_FALKOR";

pub(crate) fn falkor_endpoint() -> String {
    std::env::var("CMDB_VIEW_FALKOR_ENDPOINT")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned())
}

/// Availability gate for every test that needs a native FalkorDB graph.
/// `Ok(true)`: reachable. Unreachable: `Ok(false)` with a stderr skip reason,
/// unless `AWS_INVENTORY_REQUIRE_FALKOR=1`, which fails loudly.
pub(crate) async fn falkor_available() -> TestResult<bool> {
    let endpoint = falkor_endpoint();
    let probe = async {
        let client = redis::Client::open(endpoint.clone())?;
        let mut connection = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING")
            .query_async::<String>(&mut connection)
            .await?;
        Ok::<_, Box<dyn Error>>(())
    };
    let reason = match tokio::time::timeout(Duration::from_millis(500), probe).await {
        Ok(Ok(())) => return Ok(true),
        Ok(Err(error)) => error.to_string(),
        Err(_) => "no response within 500ms".to_owned(),
    };
    if std::env::var(REQUIRE_FALKOR).is_ok_and(|value| value == "1") {
        return Err(format!(
            "FalkorDB unreachable at {endpoint} ({reason}); {REQUIRE_FALKOR}=1 requires a running server"
        )
        .into());
    }
    eprintln!(
        "skipping native FalkorDB test: unreachable at {endpoint} ({reason}); set {REQUIRE_FALKOR}=1 to fail instead"
    );
    Ok(false)
}

impl NativeGraph {
    pub async fn new(edges: &[cmdb_view::GraphEdge]) -> TestResult<Self> {
        let endpoint = falkor_endpoint();
        let stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let name = format!("ui_worker_{}_{}", std::process::id(), stamp);
        let client = redis::Client::open(endpoint.clone())?;
        let graph = Self {
            endpoint,
            name,
            client,
            deleted: std::cell::Cell::new(false),
        };
        super::super::isolation::register_graph(&graph.name)?;
        let mut nodes = Vec::new();
        for node in edges.iter().flat_map(|edge| [&edge.from, &edge.to]) {
            if !nodes.contains(node) {
                nodes.push(node.clone());
            }
        }
        let mut clauses = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                Ok(format!(
                    "(n{index}:Resource {{key:'{}',account:'{}',region:'{}',kind:'{}',id:'{}'}})",
                    digest(node)?,
                    node.account,
                    node.region,
                    node.kind,
                    node.id
                ))
            })
            .collect::<TestResult<Vec<_>>>()?;
        for edge in edges {
            let from = nodes
                .iter()
                .position(|node| node == &edge.from)
                .ok_or("missing native source")?;
            let to = nodes
                .iter()
                .position(|node| node == &edge.to)
                .ok_or("missing native target")?;
            clauses.push(format!(
                "(n{from})-[:LINK {{relation:'{}',provenance:'{}'}}]->(n{to})",
                edge.relation, edge.provenance
            ));
        }
        graph
            .query(&format!("CREATE {}", clauses.join(",")))
            .await?;
        Ok(graph)
    }

    pub async fn query(&self, query: &str) -> TestResult<redis::Value> {
        tokio::time::timeout(Duration::from_secs(3), async {
            let mut connection = self.client.get_multiplexed_async_connection().await?;
            Ok(redis::cmd("GRAPH.QUERY")
                .arg(&self.name)
                .arg(query)
                .query_async::<redis::Value>(&mut connection)
                .await?)
        })
        .await
        .map_err(|_| "native fixture query exceeded three seconds")?
    }

    pub async fn delete(self) -> TestResult {
        tokio::time::timeout(Duration::from_secs(3), async {
            let mut connection = self.client.get_multiplexed_async_connection().await?;
            redis::cmd("GRAPH.DELETE")
                .arg(&self.name)
                .query_async::<redis::Value>(&mut connection)
                .await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await
        .map_err(|_| "native fixture deletion exceeded three seconds")??;
        self.deleted.set(true);
        Ok(())
    }
}

impl Drop for NativeGraph {
    fn drop(&mut self) {
        if !self.deleted.get()
            && let Err(error) = super::super::isolation::delete_graph(&self.name)
        {
            eprintln!(
                "failed to clean owned native fixture {}: {error}",
                self.name
            );
        }
    }
}

// Real public drain, bounded by both a deadline and a poll count; no sleep or
// injected events. Yielding only awaits the worker's actual output handshake.
pub(crate) async fn await_event(worker: &DataWorker, snapshot: bool) -> TestResult<WorkerEvent> {
    tokio::time::timeout(Duration::from_secs(20), async {
        for _ in 0..20_000_000 {
            for event in worker.drain() {
                if matches!(event, WorkerEvent::Snapshot { .. }) == snapshot {
                    return Ok(event);
                }
            }
            tokio::task::yield_now().await;
        }
        Err("worker event poll budget exhausted".into())
    })
    .await
    .map_err(|_| "real worker event deadline exceeded")?
}

pub(crate) async fn snapshot(worker: &DataWorker) -> TestResult<(Counts, Vec<Row>, u64)> {
    match await_event(worker, true).await? {
        WorkerEvent::Snapshot {
            counts,
            rows,
            total,
            error: None,
        } => Ok((counts, rows, total)),
        other => Err(format!("expected successful real snapshot, got {other:?}").into()),
    }
}

pub(crate) async fn graph(worker: &DataWorker) -> TestResult<Graph> {
    match await_event(worker, false).await? {
        WorkerEvent::Graph { graph, error: None } => Ok(graph),
        other => Err(format!("expected successful real graph, got {other:?}").into()),
    }
}

pub(crate) fn assert_edges(graph: &Graph, edges: &[cmdb_view::GraphEdge]) -> TestResult {
    let mut actual = graph.edges.clone();
    let mut expected = edges.iter().map(ui_edge).collect::<TestResult<Vec<_>>>()?;
    let sort = |edge: &crate::data::Edge| {
        (
            edge.from.clone(),
            edge.to.clone(),
            edge.label.clone(),
            edge.provenance.clone(),
        )
    };
    actual.sort_by_key(sort);
    expected.sort_by_key(sort);
    assert_eq!(actual, expected);
    Ok(())
}
