use super::*;
use inventory_core::inventory::{Kind, ResourceId};

type Node = (String, String, String, String);

struct NameCase {
    operation: &'static str,
    items: &'static str,
    prefix: &'static str,
    kind: &'static str,
    path: &'static str,
}

const CASES: [NameCase; 2] = [
    NameCase { operation: "ListCustomMetrics", items: "metricNames", prefix: "custommetric",
        kind: "AWS::IoT::CustomMetric", path: "/custom-metrics" },
    NameCase { operation: "ListDimensions", items: "dimensionNames", prefix: "dimension",
        kind: "AWS::IoT::Dimension", path: "/dimensions" },
];

#[tokio::test]
async fn given_name_only_iot_roots_when_two_pages_resume_then_canonical_graph_nodes_keep_account_and_region() -> TestResult {
    let catalog = fixture::catalog()?;
    let mut scenario = Scenario::new(&catalog, "iot-name-only-pagination").await?;
    let mut expected = BTreeSet::new();
    for account in ["123456789012", "210987654321"] {
        for region in ["us-east-1", "eu-west-1"] {
            scenario.account = AccountId::parse(account)?;
            scenario.home = RegionId::parse(region)?;
            for case in &CASES {
                scenario.seed(fixture::plan(&catalog, "iot", case.operation)?).await?;
                for name in ["shared", "page-one", "page-two"] {
                    expected.insert((account.to_owned(), region.to_owned(), case.kind.to_owned(),
                        format!("arn:aws:iot:{region}:{account}:{}/{name}", case.prefix)));
                }
            }
            let mut pages = BTreeSet::new();
            for _ in 0..4 {
                let lease = scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.ok_or("IoT name page absent")?;
                assert_eq!(lease.scope().service().as_str(), "iot");
                assert_eq!(lease.scope().account(), &scenario.account);
                assert_eq!(lease.scope().region(), &Region::Regional(scenario.home.clone()));
                let case = CASES.iter().find(|case| case.operation == lease.scope().operation().as_str())
                    .ok_or("unexpected IoT content/detail work")?;
                let plan = fixture::plan(&catalog, "iot", case.operation)?;
                let second_page = lease.token().is_some();
                assert!(pages.insert((case.operation, second_page)));
                let (request, wire) = scenario.prepared_request(plan, &lease)?;
                assert_eq!(wire.method(), http::Method::GET);
                assert!(wire.body().is_empty());
                assert_eq!(wire.uri().path(), case.path);
                assert_eq!(wire.uri().query(), second_page.then_some("nextToken=next-page"));
                assert_eq!(wire.uri().host(), Some(format!("iot.{region}.amazonaws.com").as_str()));
                scenario.apply(&catalog, plan, lease, &request, response(case, second_page)).await?;
            }
            assert_eq!(pages.len(), 4);
            assert!(scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.is_none());
        }
    }
    assert_durable_nodes(scenario, expected).await
}

fn response(case: &NameCase, second_page: bool) -> Json {
    let mut fields = serde_json::Map::new();
    fields.insert(case.items.to_owned(), json!(["shared", if second_page { "page-two" } else { "page-one" }]));
    if !second_page { fields.insert("nextToken".to_owned(), json!("next-page")); }
    Json::Object(fields)
}

async fn assert_durable_nodes(scenario: Scenario<'_, '_>, expected: BTreeSet<Node>) -> TestResult {
    let path = scenario.directory.path().join("inventory.db");
    drop(scenario.store);
    let store = BinaryStore::open(&path).await?;
    let reader = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path).read_only(true)).await?;
    let nodes: Vec<Node> = sqlx::query_as("SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id")
        .fetch_all(&reader).await?;
    assert_eq!(nodes.len(), 24);
    assert_eq!(nodes.iter().cloned().collect::<BTreeSet<_>>(), expected);
    let scopes: Vec<Node> = sqlx::query_as("SELECT account,region,kind,id FROM resource_scopes ORDER BY account,region,kind,id")
        .fetch_all(&reader).await?;
    assert_eq!(scopes, nodes);
    let facets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets").fetch_one(&reader).await?;
    assert_eq!(facets, 24);
    for (account, region, kind, id) in nodes {
        let account = AccountId::parse(&account)?;
        let region = RegionId::parse(&region)?;
        record::trusted_arn::validate_arn(&id, Some(&record::IdentityContext { account: &account, region: &region }))?;
        let key = ResourceKey { account, region: Region::Regional(region), kind: Kind::parse(&kind)?, id: ResourceId::parse(&id)? };
        assert!(store.projection_resource(&key).await?.is_some());
    }
    reader.close().await;
    Ok(())
}

#[test]
fn given_native_iot_scalar_names_without_trusted_request_scope_then_identity_fails_closed() -> TestResult {
    let catalog = fixture::catalog()?;
    for case in &CASES {
        let plan = fixture::plan(&catalog, "iot", case.operation)?;
        let response = fixture::decode(plan, &serde_json::to_vec(&response(case, false))?)?;
        let rows = projected_items(plan, &response)?;
        assert_eq!(rows.len(), 2);
        for row in rows { assert!(record::resource_id(&row, plan.identity(), None).is_err()); }
    }
    Ok(())
}
