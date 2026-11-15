use super::*;
use std::collections::BTreeSet;

const WORKSPACES: [&str; 2] = ["factory-a", "factory-b"];
const IDS: [&str; 2] = ["66666666-6666-4666-8666-666666666666", "77777777-7777-4777-8777-777777777777"];

#[tokio::test]
async fn given_both_dataset_sources_and_unattached_series_when_real_pages_resume_then_every_definition_is_queryable_without_execution_calls() -> TestResult {
    // Given real embedded plans: applications are an independent root, not a workspace child.
    let catalog = catalog()?;
    let mut scenario = Scenario::new(&catalog, "sitewise-definitions").await?;
    for operation in ["ListWorkspaces", "ListApplications", "ListDatasets", "ListTimeSeries"] {
        scenario.seed(plan(&catalog, "iotsitewise", operation)?).await?;
    }
    let mut requests = BTreeSet::new();
    // When durable empty first pages resume, both source branches and workspace children run.
    for _ in 0..24 {
        let Some(lease) = scenario.store.claim(&scenario.sweep, 1001, 10_000).await? else { break; };
        let operation = lease.scope().operation().as_str().to_owned();
        let plan = plan(&catalog, "iotsitewise", &operation)?;
        let (request, wire) = scenario.request(plan, &lease)?;
        assert_eq!(wire.method(), http::Method::GET);
        let later = lease.token().is_some();
        if later { assert_eq!(query(&wire, "nextToken")?, "resume-definitions"); }
        let source = if operation == "ListDatasets" {
            let source = query(&wire, "sourceType")?.to_owned();
            assert_eq!(["KENDRA", "SITEWISE"].contains(&source.as_str()), true);
            assert_eq!(lease.scope().branch(), Some(source.as_str()));
            assert_eq!(wire.uri().query().map(|query| query.contains("workspaceName=") || query.contains("datasetType=")), Some(false));
            source
        } else { String::new() };
        let workspace = if matches!(operation.as_str(), "ListPipelines" | "ListTasks") {
            let name = wire.uri().path().split('/').nth(2).ok_or("workspace path label absent")?.to_owned();
            assert_eq!(WORKSPACES.contains(&name.as_str()), true);
            assert_eq!(lease.scope().parent().ok_or("definition lost workspace")?.id.as_str(), arn("iotsitewise", &format!("workspace/{name}")));
            name
        } else { String::new() };
        if operation == "ListTimeSeries" {
            assert_eq!(wire.uri().query().is_none_or(|query| ["assetId=", "aliasPrefix=", "timeSeriesType=", "workspaceName="].iter().all(|filter| !query.contains(filter))), true);
        }
        assert_eq!(requests.insert((operation.clone(), source.clone(), workspace.clone(), later)), true);
        let (items, rows) = summaries(&operation, &source, &workspace)?;
        let response = if later { json!({items:rows}) } else { json!({items:[],"nextToken":"resume-definitions"}) };
        let keys = scenario.apply(&catalog, plan, lease, &request, response).await?;
        if later && operation == "ListTimeSeries" {
            assert_eq!(keys.iter().map(|key| key.id.as_str()).collect::<BTreeSet<_>>(), BTreeSet::from([
                "arn:aws:iotsitewise:us-east-1:123456789012:time-series/66666666-6666-4666-8666-666666666666",
                "arn:aws:iotsitewise:us-east-1:123456789012:time-series/77777777-7777-4777-8777-777777777777"]));
        }
    }
    // Then every producer has visible SQL rows and canonical graph nodes after reopen.
    assert_eq!(requests.len(), 18);
    assert_eq!(scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.is_none(), true);
    let reader = scenario.reader().await?;
    let counts: Vec<(String, i64)> = sqlx::query_as("SELECT kind,COUNT(*) FROM resource_facets GROUP BY kind ORDER BY kind").fetch_all(&reader).await?;
    assert_eq!(counts, vec![("AWS::ApiOnly::IoTSiteWiseTimeSeries".to_owned(),2),
        ("AWS::IoTSiteWise::Application".to_owned(),2),("AWS::IoTSiteWise::Dataset".to_owned(),2),
        ("AWS::IoTSiteWise::Pipeline".to_owned(),2),("AWS::IoTSiteWise::Task".to_owned(),2),("AWS::IoTSiteWise::Workspace".to_owned(),2)]);
    let dataset_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM resource_facets WHERE kind='AWS::IoTSiteWise::Dataset' ORDER BY id").fetch_all(&reader).await?;
    assert_eq!(dataset_ids, IDS.iter().map(|id| arn("iotsitewise", &format!("dataset/{id}"))).collect::<Vec<_>>());
    let edges = scenario.store.projection_edges(None, 30).await?;
    assert_eq!(edges.edges.len(), 6);
    for edge in edges.edges {
        assert_eq!(edge.relation.as_str(), "IN_WORKSPACE");
        assert_eq!(edge.to.kind.as_str(), "AWS::IoTSiteWise::Workspace");
        let workspace = edge.to.id.as_str().rsplit('/').next().ok_or("workspace suffix absent")?;
        assert_eq!(edge.from.id.as_str().contains(workspace), true);
    }
    let nodes = scenario.store.projection_resources(None, 30).await?;
    assert_eq!(nodes.len(), 12);
    reader.close().await;
    Ok(())
}

fn summaries(operation: &str, source: &str, workspace: &str) -> TestResult<(&'static str, Vec<Json>)> {
    Ok(match operation {
        "ListWorkspaces" => ("workspaceSummaries", WORKSPACES.iter().map(|name| json!({"name":name,"arn":arn("iotsitewise", &format!("workspace/{name}")),"status":{"state":"ACTIVE"},"createdAt":1.0,"updatedAt":2.0})).collect()),
        "ListApplications" => ("applications", WORKSPACES.iter().zip(IDS).map(|(name, id)| json!({"id":id,"arn":arn("iotsitewise", &format!("application/app-{name}")),"name":format!("app-{name}"),"workspaceName":name,"status":"ACTIVE","createdAt":1.0})).collect()),
        "ListDatasets" => {
            let id = match source { "KENDRA" => IDS.first(), "SITEWISE" => IDS.get(1), _ => None }.ok_or("unknown dataset source")?;
            ("datasetSummaries", vec![json!({"id":id,"arn":arn("iotsitewise", &format!("dataset/{id}")),"name":source,"description":"metadata only","sourceType":source,"creationDate":1.0,"lastUpdateDate":2.0,"status":{"state":"ACTIVE"},"enrichmentStatus":{"video":{"status":"NOT_ENRICHED","lastEnrichedAt":2.0}}})])
        },
        "ListPipelines" => ("pipelineSummaries", vec![json!({"pipelineName":"shared","pipelineArn":arn("iotsitewise", &format!("workspace/{workspace}/pipeline/shared")),"version":"1","status":{"state":"ACTIVE"},"createdAt":1.0,"updatedAt":2.0})]),
        "ListTasks" => ("taskSummaries", vec![json!({"taskName":"shared","taskArn":arn("iotsitewise", &format!("workspace/{workspace}/task/shared")),"version":"1","status":{"state":"ACTIVE"},"createdAt":1.0,"updatedAt":2.0})]),
        "ListTimeSeries" => ("TimeSeriesSummaries", IDS.iter().map(|id| json!({"timeSeriesId":id,"timeSeriesArn":arn("iotsitewise", &format!("time-series/{id}")),"dataType":"DOUBLE","timeSeriesCreationDate":1.0,"timeSeriesLastUpdateDate":2.0})).collect()),
        _ => return Err(format!("forbidden definition/content/execution call {operation}").into()),
    })
}
