use super::*;
use std::collections::BTreeSet;

const PORTALS: [&str; 2] = ["11111111-1111-4111-8111-111111111111", "22222222-2222-4222-8222-222222222222"];
const PROJECTS: [&str; 2] = ["33333333-3333-4333-8333-333333333333", "44444444-4444-4444-8444-444444444444"];
const DASHBOARD: &str = "55555555-5555-4555-8555-555555555555";
const POLICY: &str = "66666666-6666-4666-8666-666666666666";

#[tokio::test]
async fn given_two_portals_when_empty_child_pages_resume_then_monitor_resources_and_guarded_access_policies_survive_sqlite_reopen() -> TestResult {
    // Given pinned embedded policies and a real durable child queue, not synthetic plans.
    let catalog = catalog()?;
    let mut scenario = Scenario::new(&catalog, "sitewise-monitor-lineage").await?;
    scenario.seed(plan(&catalog, "iotsitewise", "ListPortals")?).await?;
    let mut requests = BTreeSet::new();
    // When each first page is empty with a cursor, parent-bound work must still resume.
    for _ in 0..24 {
        let Some(lease) = scenario.store.claim(&scenario.sweep, 1001, 10_000).await? else { break; };
        let operation = lease.scope().operation().as_str().to_owned();
        let plan = plan(&catalog, "iotsitewise", &operation)?;
        let (request, wire) = scenario.request(plan, &lease)?;
        let later = lease.token().is_some();
        assert_eq!(wire.method(), http::Method::GET);
        if later { assert_eq!(query(&wire, "nextToken")?, "resume-monitor"); }
        let (items, rows, parent) = match operation.as_str() {
            "ListPortals" => ("portalSummaries", PORTALS.iter().map(|id| json!({"id":id,"name":id,"startUrl":"https://monitor.example.test","status":{"state":"ACTIVE"}})).collect::<Vec<_>>(), String::new()),
            "ListProjects" => {
                let portal = query(&wire, "portalId")?.to_owned();
                let ordinal = PORTALS.iter().position(|id| *id == portal).ok_or("unknown bound portal")?;
                assert_eq!(lease.scope().parent().ok_or("project lost portal")?.id.as_str(), portal);
                let project = PROJECTS.get(ordinal).ok_or("project fixture absent")?;
                ("projectSummaries", vec![json!({"id":project,"name":"project","description":"metadata","creationDate":1.0,"lastUpdateDate":2.0})], portal)
            },
            "ListDashboards" => {
                let project = query(&wire, "projectId")?.to_owned();
                assert_eq!(PROJECTS.contains(&project.as_str()), true);
                let parent = lease.scope().parent().ok_or("dashboard lost project")?;
                assert_eq!(parent.kind.as_str(), "AWS::IoTSiteWise::Project");
                assert_eq!(parent.id.as_str().ends_with(&format!("{}:{project}", project.len())), true);
                ("dashboardSummaries", vec![json!({"id":DASHBOARD,"name":"shared-dashboard","creationDate":1.0,"lastUpdateDate":2.0})], project)
            },
            "ListAccessPolicies" => {
                let resource_type = query(&wire, "resourceType")?;
                let resource_id = query(&wire, "resourceId")?;
                assert_eq!(lease.scope().branch(), Some(resource_type));
                let bound = lease.scope().parent().ok_or("policy lost resource owner")?;
                match resource_type {
                    "PORTAL" => {
                        assert_eq!(PORTALS.contains(&resource_id), true);
                        assert_eq!(bound.kind.as_str(), "AWS::IoTSiteWise::Portal");
                        assert_eq!(bound.id.as_str(), resource_id);
                    },
                    "PROJECT" => {
                        assert_eq!(PROJECTS.contains(&resource_id), true);
                        assert_eq!(bound.kind.as_str(), "AWS::IoTSiteWise::Project");
                        assert_eq!(bound.id.as_str().ends_with(&format!("{}:{resource_id}", resource_id.len())), true);
                    },
                    _ => return Err("unreviewed policy resource branch".into()),
                }
                assert_eq!(wire.uri().query().is_none_or(|query| ["identityType=", "identityId=", "iamArn="].iter().all(|filter| !query.contains(filter))), true);
                let key = resource_type.to_ascii_lowercase();
                ("accessPolicySummaries", vec![json!({"id":POLICY,
                    "identity":{"iamUser":{"arn":"arn:aws:iam::123456789012:user/inventory-reference"}},
                    "resource":{key:{"id":resource_id}},"permission":"VIEWER","creationDate":1.0,"lastUpdateDate":2.0})],
                    format!("{resource_type}/{resource_id}"))
            },
            _ => return Err(format!("forbidden unexpected Monitor child {operation}").into()),
        };
        assert_eq!(requests.insert((operation.clone(), parent.clone(), later)), true);
        let response = if later { json!({items:rows}) } else { json!({items:[],"nextToken":"resume-monitor"}) };
        let keys = scenario.apply(&catalog, plan, lease, &request, response).await?;
        if later && operation == "ListDashboards" {
            assert_eq!(keys.first().ok_or("dashboard was not projected")?.id.as_str(), format!("{}:{parent}|{}:{DASHBOARD}", parent.len(), DASHBOARD.len()));
        }
        if later && operation == "ListAccessPolicies" {
            let (kind, id) = parent.split_once('/').ok_or("policy scope fixture malformed")?;
            assert_eq!(keys.first().ok_or("policy was not projected")?.id.as_str(),
                format!("{}:{kind}|{}:{id}|{}:{POLICY}", kind.len(), id.len(), POLICY.len()));
        }
    }
    // Then every page/parent request ran, both identically named dashboards remain queryable,
    // and the real relationship resolver links each child to its canonical owner.
    assert_eq!(requests.len(), 18);
    assert_eq!(scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.is_none(), true);
    let reader = scenario.reader().await?;
    let counts: Vec<(String, i64)> = sqlx::query_as("SELECT kind,COUNT(*) FROM resource_facets GROUP BY kind ORDER BY kind").fetch_all(&reader).await?;
    assert_eq!(counts, vec![("AWS::IoTSiteWise::AccessPolicy".to_owned(),4),("AWS::IoTSiteWise::Dashboard".to_owned(),2),("AWS::IoTSiteWise::Portal".to_owned(),2),("AWS::IoTSiteWise::Project".to_owned(),2)]);
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM resource_facets WHERE kind='AWS::IoTSiteWise::Dashboard' ORDER BY id").fetch_all(&reader).await?;
    assert_eq!(ids, PROJECTS.iter().map(|project| format!("{}:{project}|{}:{DASHBOARD}", project.len(), DASHBOARD.len())).collect::<Vec<_>>());
    let policy_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM resource_facets WHERE kind='AWS::IoTSiteWise::AccessPolicy' ORDER BY id").fetch_all(&reader).await?;
    let expected_policies = [("PORTAL", PORTALS), ("PROJECT", PROJECTS)].into_iter().flat_map(|(kind, ids)| {
        ids.into_iter().map(move |id| format!("{}:{kind}|{}:{id}|{}:{POLICY}", kind.len(), id.len(), POLICY.len()))
    }).collect::<Vec<_>>();
    assert_eq!(policy_ids, expected_policies);
    let edges = scenario.store.projection_edges(None, 20).await?;
    assert_eq!(edges.edges.len(), 8);
    for edge in edges.edges {
        match edge.relation.as_str() {
            "IN_PORTAL" => {
                let prefix = if edge.from.kind.as_str() == "AWS::IoTSiteWise::AccessPolicy" { "6:PORTAL|" } else { "" };
                assert_eq!(edge.from.id.as_str().starts_with(&format!("{prefix}{}:{}|", edge.to.id.as_str().len(), edge.to.id.as_str())), true);
            },
            "IN_PROJECT" => {
                let project = edge.to.id.as_str().rsplit('|').next().and_then(|part| part.split_once(':')).ok_or("project tuple malformed")?.1;
                let expected = if edge.from.kind.as_str() == "AWS::IoTSiteWise::AccessPolicy" {
                    format!("7:PROJECT|{}:{project}|{}:{POLICY}", project.len(), POLICY.len())
                } else {
                    format!("{}:{project}|{}:{DASHBOARD}", project.len(), DASHBOARD.len())
                };
                assert_eq!(edge.from.id.as_str(), expected);
            },
            _ => return Err("unexpected Monitor graph relation".into()),
        }
    }
    reader.close().await;
    Ok(())
}

#[test]
fn given_same_project_id_when_parent_request_changes_then_identity_is_parent_qualified_and_missing_context_is_rejected() -> TestResult {
    use crate::sweep_runtime::{input, record};
    use inventory_core::{inventory::DiscoveryContext, record::Value};
    let catalog = catalog()?;
    let plan = plan(&catalog, "iotsitewise", "ListProjects")?;
    let response = crate::sweep_runtime::record::policy_dfgk_tests::fixture::decode(plan, &serde_json::to_vec(&json!({"projectSummaries":[{"id":PROJECTS.first().ok_or("project absent")?,"name":"same"}]}))?)?;
    let rows = record::item_records(&response, plan.items())?;
    let row = rows.first().ok_or("project row absent")?;
    let path = plan.bindings().first().ok_or("portal binding absent")?.path.clone();
    let keys = PORTALS.iter().map(|portal| {
        let request = input::merge_input(&inventory_core::record::Record::encode(&Value::Map(Default::default()))?, &[input::InputField { path:path.clone(), value:Value::Text((*portal).to_owned()) }])?;
        Ok(record::resource_id_with_context(row, plan.identity(), None, &DiscoveryContext::empty(), &request)?.as_str().to_owned())
    }).collect::<TestResult<Vec<_>>>()?;
    assert_ne!(keys.first(), keys.get(1));
    assert_eq!(keys.len(), 2);
    assert_eq!(matches!(record::resource_id(row, plan.identity(), None), Err(crate::sweep_runtime::SweepError::Operation("identity requires actual request context"))), true);
    Ok(())
}
