use super::*;
use std::collections::BTreeSet;

#[tokio::test]
async fn given_two_workspaces_when_scene_and_sync_job_pages_resume_then_same_names_remain_distinct_and_attached_without_content_fetches() -> TestResult {
    // Given pinned summary-only plans and a real SQLite work queue.
    let catalog = catalog()?;
    let mut scenario = Scenario::new(&catalog, "twinmaker-scene-sync").await?;
    scenario.seed(plan(&catalog, "iottwinmaker", "ListWorkspaces")?).await?;
    let mut requests = BTreeSet::new();
    // When each empty initial page checkpoints, the next request carries both cursor and workspace.
    for _ in 0..12 {
        let Some(lease) = scenario.store.claim(&scenario.sweep, 1001, 10_000).await? else { break; };
        let operation = lease.scope().operation().as_str().to_owned();
        let plan = plan(&catalog, "iottwinmaker", &operation)?;
        let (request, wire) = scenario.request(plan, &lease)?;
        assert_eq!(wire.method(), http::Method::POST);
        let body: Json = serde_json::from_slice(wire.body())?;
        let later = lease.token().is_some();
        assert_eq!(body.get("nextToken").and_then(Json::as_str), later.then_some("resume-twin"));
        let workspace = if operation == "ListWorkspaces" { String::new() } else {
            let workspace = wire.uri().path().split('/').nth(2).ok_or("workspace path absent")?.to_owned();
            assert_eq!(["factory-a", "factory-b"].contains(&workspace.as_str()), true);
            assert_eq!(lease.scope().parent().ok_or("TwinMaker child lost workspace")?.id.as_str(), arn("iottwinmaker", &format!("workspace/{workspace}")));
            workspace
        };
        assert_eq!(requests.insert((operation.clone(), workspace.clone(), later)), true);
        let (items, rows) = match operation.as_str() {
            "ListWorkspaces" => ("workspaceSummaries", ["factory-a", "factory-b"].iter().map(|id| json!({"workspaceId":id,"arn":arn("iottwinmaker", &format!("workspace/{id}")),"creationDateTime":1.0,"updateDateTime":2.0})).collect::<Vec<_>>()),
            "ListScenes" => {
                assert_eq!(wire.uri().path(), format!("/workspaces/{workspace}/scenes-list"));
                ("sceneSummaries", vec![json!({"sceneId":"shared","arn":arn("iottwinmaker", &format!("workspace/{workspace}/scene/shared")),"contentLocation":format!("s3://scene-references/{workspace}/shared.json"),"creationDateTime":1.0,"updateDateTime":2.0})])
            },
            "ListSyncJobs" => {
                assert_eq!(wire.uri().path(), format!("/workspaces/{workspace}/sync-jobs-list"));
                ("syncJobSummaries", vec![json!({"syncSource":"SITEWISE","workspaceId":workspace,"arn":arn("iottwinmaker", &format!("workspace/{workspace}/sync-job/SITEWISE")),"status":{"state":"ACTIVE"},"creationDateTime":1.0,"updateDateTime":2.0})])
            },
            _ => return Err(format!("unexpected TwinMaker content/template call {operation}").into()),
        };
        let response = if later { json!({items:rows}) } else { json!({items:[],"nextToken":"resume-twin"}) };
        scenario.apply(&catalog, plan, lease, &request, response).await?;
    }
    // Then both workspaces' same-named resources are independent SQL rows and exact graph nodes.
    assert_eq!(requests.len(), 10);
    assert_eq!(scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.is_none(), true);
    let reader = scenario.reader().await?;
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT kind,id FROM resource_facets ORDER BY kind,id").fetch_all(&reader).await?;
    let expected = ["Scene", "SyncJob", "Workspace"].iter().flat_map(|kind| {
        ["factory-a", "factory-b"].iter().map(move |workspace| {
            let resource = match *kind {
                "Scene" => format!("workspace/{workspace}/scene/shared"),
                "SyncJob" => format!("workspace/{workspace}/sync-job/SITEWISE"),
                _ => format!("workspace/{workspace}"),
            };
            (format!("AWS::IoTTwinMaker::{kind}"), arn("iottwinmaker", &resource))
        })
    }).collect::<Vec<_>>();
    assert_eq!(rows, expected);
    let edges = scenario.store.projection_edges(None, 20).await?;
    assert_eq!(edges.edges.len(), 4);
    for edge in edges.edges {
        assert_eq!(edge.relation.as_str(), "IN_WORKSPACE");
        assert_eq!(edge.to.kind.as_str(), "AWS::IoTTwinMaker::Workspace");
        assert_eq!(edge.from.id.as_str().starts_with(&format!("{}/", edge.to.id.as_str())), true);
    }
    assert_eq!(scenario.store.projection_resources(None, 20).await?.len(), 6);
    reader.close().await;
    Ok(())
}

#[test]
fn given_metadata_inventory_when_catalog_is_compiled_then_content_and_provider_template_calls_are_not_callable() -> TestResult {
    let catalog = catalog()?;
    for (service, operations) in [
        ("iotsitewise", &["GetAssetPropertyValue", "ExecuteQuery", "DescribeDashboard", "ListPipelineExecutions"][..]),
        ("iottwinmaker", &["GetScene", "GetPropertyValue", "ExecuteQuery", "ListComponentTypes"][..]),
    ] {
        for operation in operations {
            assert_eq!(catalog.lookup(service, operation).is_none(), true, "{service}.{operation} must not run for metadata inventory");
        }
    }
    Ok(())
}
