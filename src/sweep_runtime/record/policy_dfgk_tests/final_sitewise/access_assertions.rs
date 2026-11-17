use super::*;

const PORTAL: &str = "11111111-1111-4111-8111-111111111111";
const PROJECT: &str = "33333333-3333-4333-8333-333333333333";
const FIRST_POLICY: &str = "66666666-6666-4666-8666-666666666666";
const SECOND_POLICY: &str = "77777777-7777-4777-8777-777777777777";

#[derive(Clone, Copy, Debug)]
enum ReturnedResource { Equal, ForeignId, MissingResource, MissingId, WrongVariant, BothVariants, EmptyOpposite, NullOpposite, NullExpected, WrongExpectedType, WrongIdType }

fn summary(id: &str, resource: &str, owner: &str) -> Json {
    json!({"id":id,"permission":"VIEWER","identity":{"iamUser":{"arn":"arn:aws:iam::123456789012:user/inventory-reference"}},
        "resource":{(resource):{"id":owner}},"creationDate":1.0,"lastUpdateDate":2.0})
}

async fn target_lease(scenario: &mut Scenario<'_, '_>, catalog: &inventory_core::catalog::runtime::RuntimeCatalog<'_>, branch: &str)
    -> TestResult<inventory_engine::inventory::WorkLease> {
    for _ in 0..8 {
        let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("bound access-policy work absent")?;
        let operation = lease.scope().operation().as_str();
        if operation == "ListAccessPolicies" && lease.scope().branch() == Some(branch) { return Ok(lease); }
        let current = plan(catalog, "iotsitewise", operation)?;
        let (request, _) = scenario.request(current, &lease)?;
        let body = match operation {
            "ListProjects" => json!({"projectSummaries":[{"id":PROJECT,"name":"owned project","creationDate":1.0,"lastUpdateDate":2.0}]}),
            "ListDashboards" => json!({"dashboardSummaries":[]}),
            "ListAccessPolicies" => json!({"accessPolicySummaries":[]}),
            _ => return Err("unreviewed Monitor detail or data request scheduled".into()),
        };
        scenario.apply(catalog, current, lease, &request, body).await?;
    }
    Err("target access-policy branch was never reached".into())
}

#[tokio::test]
async fn given_portal_and_project_bound_policies_when_native_resource_contradicts_branch_then_no_partial_rows_or_foreign_edges_commit() -> TestResult {
    let catalog = catalog()?;
    let portals = plan(&catalog, "iotsitewise", "ListPortals")?;
    let policies = plan(&catalog, "iotsitewise", "ListAccessPolicies")?;
    for branch in ["PORTAL", "PROJECT"] {
        for variant in [ReturnedResource::Equal, ReturnedResource::ForeignId, ReturnedResource::MissingResource,
            ReturnedResource::MissingId, ReturnedResource::WrongVariant, ReturnedResource::BothVariants,
            ReturnedResource::EmptyOpposite, ReturnedResource::NullOpposite, ReturnedResource::NullExpected,
            ReturnedResource::WrongExpectedType, ReturnedResource::WrongIdType] {
            // Given durable Monitor parents and a real branch-specific native child request.
            let mut scenario = Scenario::new(&catalog, &format!("policy-resource-{branch}-{variant:?}")).await?;
            scenario.seed(portals).await?;
            let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("portal lease absent")?;
            let (request, _) = scenario.request(portals, &lease)?;
            scenario.apply(&catalog, portals, lease, &request, json!({"portalSummaries":[{"id":PORTAL,"name":"owned portal","status":{"state":"ACTIVE"}}]})).await?;
            let lease = target_lease(&mut scenario, &catalog, branch).await?;
            let bound_parent = lease.scope().parent().ok_or("access policy lost bound canonical parent")?.clone();
            let (request, wire) = scenario.request(policies, &lease)?;
            assert_eq!(wire.method(), http::Method::GET);
            assert_eq!(query(&wire, "resourceType")?, branch);
            let owner = query(&wire, "resourceId")?;
            assert_eq!(owner, if branch == "PORTAL" { PORTAL } else { PROJECT });
            let resource = if branch == "PORTAL" { "portal" } else { "project" };
            let opposite = if branch == "PORTAL" { "project" } else { "portal" };
            let mut second = summary(SECOND_POLICY, resource, owner);
            let object = second.as_object_mut().ok_or("summary is not an object")?;
            let replacement = match variant {
                ReturnedResource::Equal => None,
                ReturnedResource::ForeignId => Some(json!({(resource):{"id":"99999999-9999-4999-8999-999999999999"}})),
                ReturnedResource::MissingResource => { object.remove("resource"); None },
                ReturnedResource::MissingId => Some(json!({(resource):{}})),
                ReturnedResource::WrongVariant => Some(json!({(opposite):{"id":owner}})),
                ReturnedResource::BothVariants => Some(json!({(resource):{"id":owner},(opposite):{"id":owner}})),
                ReturnedResource::EmptyOpposite => Some(json!({(resource):{"id":owner},(opposite):{}})),
                ReturnedResource::NullOpposite => Some(json!({(resource):{"id":owner},(opposite):null})),
                ReturnedResource::NullExpected => Some(json!({(resource):null})),
                ReturnedResource::WrongExpectedType => Some(json!({(resource):"not-a-structure"})),
                ReturnedResource::WrongIdType => Some(json!({(resource):{"id":7}})),
            };
            if let Some(replacement) = replacement { object.insert("resource".to_owned(), replacement); }
            // When an otherwise valid first row precedes a contradictory native resource row.
            let result = scenario.apply(&catalog, policies, lease, &request,
                json!({"accessPolicySummaries":[summary(FIRST_POLICY, resource, owner),second]})).await;
            let accepted = matches!(variant, ReturnedResource::Equal | ReturnedResource::NullOpposite);
            if accepted { result?; } else { assert!(result.is_err(), "{branch} admitted {variant:?}"); }
            scenario = scenario.reopen().await?;
            // Then reopened SQLite contains both correct owner edges or no policy facet/edge.
            let reader = scenario.reader().await?;
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE kind='AWS::IoTSiteWise::AccessPolicy'").fetch_one(&reader).await?;
            assert_eq!(count, if accepted { 2 } else { 0 }, "{branch} {variant:?}");
            let edges: Vec<(String, String, String)> = sqlx::query_as("SELECT to_kind,to_id,relation FROM edges WHERE from_kind='AWS::IoTSiteWise::AccessPolicy'").fetch_all(&reader).await?;
            assert_eq!(edges.len(), if accepted { 2 } else { 0 });
            for (kind, id, relation) in edges {
                assert_eq!(kind, bound_parent.kind.as_str());
                assert_eq!(id, bound_parent.id.as_str());
                assert_eq!(relation, if branch == "PORTAL" { "IN_PORTAL" } else { "IN_PROJECT" });
            }
            let parents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE kind='AWS::IoTSiteWise::Portal'").fetch_one(&reader).await?;
            assert_eq!(parents, 1, "rejected policy page erased its durable parent");
            reader.close().await;
        }
    }
    Ok(())
}
