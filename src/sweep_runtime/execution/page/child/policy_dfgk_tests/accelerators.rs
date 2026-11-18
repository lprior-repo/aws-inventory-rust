use super::*;

const FAMILIES: &[(&str, &str, &str, &str)] = &[
    (
        "standard",
        "AWS::GlobalAccelerator::Accelerator",
        "AWS::GlobalAccelerator::Listener",
        "AWS::GlobalAccelerator::EndpointGroup",
    ),
    (
        "custom",
        "AWS::ApiOnly::GlobalAcceleratorCustomRoutingAccelerator",
        "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener",
        "AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup",
    ),
];

#[tokio::test]
async fn given_standard_and_custom_accelerators_when_bound_child_pages_resume_then_global_nodes_and_parent_edges_survive_reopen()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "accelerator-routing-family-descendants").await?;
    for operation in ["ListAccelerators", "ListCustomRoutingAccelerators"] {
        scenario
            .seed(fixture::plan(&catalog, "global-accelerator", operation)?)
            .await?;
    }
    // Two roots, two standard details, four tag refreshes, eight listener pages,
    // and sixteen endpoint-group pages. Every child comes from production work().
    let mut requests = BTreeSet::new();
    for _ in 0..32 {
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10_000)
            .await?
            .ok_or("accelerator child page was lost")?;
        let operation = lease.scope().operation().as_str().to_owned();
        let plan = fixture::plan(&catalog, "global-accelerator", &operation)?;
        let (request, wire) = scenario.request(plan, &lease)?;
        let second_page = lease.token().is_some();
        if operation.ends_with("Listeners") || operation.ends_with("EndpointGroups") {
            assert_eq!(
                wire.get("NextToken").and_then(Json::as_str),
                second_page.then_some("same-next-token")
            );
            let parent = lease
                .scope()
                .parent()
                .ok_or("accelerator descendant lost parent")?;
            let field = if operation.ends_with("Listeners") {
                "AcceleratorArn"
            } else {
                "ListenerArn"
            };
            assert_eq!(text(&wire, field)?, parent.id.as_str());
            assert!(requests.insert((
                operation.clone(),
                parent.id.as_str().to_owned(),
                second_page
            )));
        }
        let body = response(&operation, &wire, second_page)?;
        scenario
            .apply(&catalog, plan, lease, &request, body)
            .await?;
    }
    assert_eq!(requests.len(), 24);
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 1001, 10_000)
            .await?
            .is_none()
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
    assert_persisted(&reader).await?;
    reader.close().await;
    drop(reopened);
    drop(directory);
    Ok(())
}

fn response(operation: &str, wire: &Json, second: bool) -> TestResult<Json> {
    match operation {
        "ListAccelerators" | "ListCustomRoutingAccelerators" => {
            let family = if operation == "ListAccelerators" {
                "standard"
            } else {
                "custom"
            };
            Ok(
                json!({"Accelerators":[{"AcceleratorArn":accelerator(family, "a")},
                {"AcceleratorArn":accelerator(family, "b")}]}),
            )
        }
        "DescribeAccelerator" => {
            Ok(json!({"Accelerator":{"AcceleratorArn":text(wire, "AcceleratorArn")?}}))
        }
        "ListTagsForResource" => Ok(json!({"Tags":[]})),
        "ListListeners" | "ListCustomRoutingListeners" => {
            let arn = format!(
                "{}/listener/{}",
                text(wire, "AcceleratorArn")?,
                page(second)
            );
            // Native listener rows have no AcceleratorArn. The join must use the real request.
            paginated(json!({"Listeners":[{"ListenerArn":arn}]}), second)
        }
        "ListEndpointGroups" | "ListCustomRoutingEndpointGroups" => {
            let arn = format!(
                "{}/endpoint-group/{}",
                text(wire, "ListenerArn")?,
                page(second)
            );
            // Region describes endpoint placement, not request routing or resource scope.
            paginated(
                json!({"EndpointGroups":[{"EndpointGroupArn":arn,"EndpointGroupRegion":"eu-west-1"}]}),
                second,
            )
        }
        _ => Err(format!("unexpected accelerator work: {operation}").into()),
    }
}

fn paginated(mut response: Json, second: bool) -> TestResult<Json> {
    if !second {
        response
            .as_object_mut()
            .ok_or("response is not an object")?
            .insert("NextToken".to_owned(), json!("same-next-token"));
    }
    Ok(response)
}

fn page(second: bool) -> &'static str {
    if second { "second" } else { "first" }
}
fn accelerator(family: &str, name: &str) -> String {
    format!("arn:aws:globalaccelerator::123456789012:accelerator/{family}-{name}")
}

type StoredEdge = (String, String, String, String, String);

async fn assert_persisted(reader: &sqlx::SqlitePool) -> TestResult {
    let mut expected_nodes = BTreeSet::new();
    let mut expected_edges = BTreeSet::new();
    for (family, accelerator_kind, listener_kind, group_kind) in FAMILIES {
        for name in ["a", "b"] {
            let accelerator = accelerator(family, name);
            expected_nodes.insert(((*accelerator_kind).to_owned(), accelerator.clone()));
            for listener_page in ["first", "second"] {
                let listener = format!("{accelerator}/listener/{listener_page}");
                expected_nodes.insert(((*listener_kind).to_owned(), listener.clone()));
                expected_edges.insert((
                    (*listener_kind).to_owned(),
                    listener.clone(),
                    (*accelerator_kind).to_owned(),
                    accelerator.clone(),
                    "IN_ACCELERATOR".to_owned(),
                ));
                for group_page in ["first", "second"] {
                    let group = format!("{listener}/endpoint-group/{group_page}");
                    expected_nodes.insert(((*group_kind).to_owned(), group.clone()));
                    expected_edges.insert((
                        (*group_kind).to_owned(),
                        group,
                        (*listener_kind).to_owned(),
                        listener.clone(),
                        "IN_LISTENER".to_owned(),
                    ));
                }
            }
        }
    }
    let nodes: Vec<(String, String)> =
        sqlx::query_as("SELECT kind,id FROM resources ORDER BY kind,id")
            .fetch_all(reader)
            .await?;
    let edges: Vec<StoredEdge> = sqlx::query_as("SELECT from_kind,from_id,to_kind,to_id,relation FROM edges ORDER BY from_kind,from_id,to_kind,to_id,relation").fetch_all(reader).await?;
    assert_eq!(nodes.into_iter().collect::<BTreeSet<_>>(), expected_nodes);
    assert_eq!(edges.into_iter().collect::<BTreeSet<_>>(), expected_edges);
    let regions: Vec<String> = sqlx::query_scalar("SELECT DISTINCT region FROM resources")
        .fetch_all(reader)
        .await?;
    assert_eq!(regions, vec![Region::Global.as_str().to_owned()]);
    Ok(())
}
