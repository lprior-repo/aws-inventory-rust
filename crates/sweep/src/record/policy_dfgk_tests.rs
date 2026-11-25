mod final_sitewise;
pub(crate) mod fixture;
mod principals;
mod trust_and_metadata;

use super::*;
use crate::input;
use fixture::{catalog, decode, observations, parent_fields, persist, plan, request, rows};
use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog},
    inventory::Region,
};
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_mixed_search_engines_when_real_requests_are_encoded_then_domains_have_disjoint_canonical_nodes()
-> TestResult {
    let catalog = catalog()?;
    let mut keys = std::collections::BTreeSet::new();
    for (service, engine, name, kind, expected_query) in [
        (
            "elasticsearch-service",
            "Elasticsearch",
            "legacy-a",
            "AWS::Elasticsearch::Domain",
            Some("engineType=Elasticsearch"),
        ),
        (
            "opensearch",
            "OpenSearch",
            "modern-a",
            "AWS::OpenSearchService::Domain",
            None,
        ),
    ] {
        let plan = plan(&catalog, service, "ListDomainNames")?;
        let request = request(plan, &[])?;
        assert_eq!(request.uri().query(), expected_query);
        let body =
            serde_json::to_vec(&json!({"DomainNames":[{"DomainName":name,"EngineType":engine}]}))?;
        let saved = persist(
            catalog.raw_bytes(),
            plan,
            observations(plan, rows(plan, &body)?)?,
        )
        .await?;
        assert_eq!(saved.len(), 1);
        let (saved_kind, id, _) = saved.first().ok_or("domain was not persisted")?;
        assert_eq!((saved_kind.as_str(), id.as_str()), (kind, name));
        keys.insert((saved_kind.clone(), id.clone()));
    }
    assert_eq!(keys.len(), 2);
    Ok(())
}

#[tokio::test]
async fn given_databrew_rows_without_optional_arns_when_collected_then_required_names_preserve_every_sibling()
-> TestResult {
    let catalog = catalog()?;
    for (operation, member, resource) in [
        ("ListJobs", "Jobs", "job"),
        ("ListProjects", "Projects", "project"),
        ("ListRulesets", "Rulesets", "ruleset"),
        ("ListSchedules", "Schedules", "schedule"),
    ] {
        let mut first = serde_json::Map::from_iter([("Name".to_owned(), json!("a"))]);
        if operation == "ListProjects" {
            first.insert("RecipeName".to_owned(), json!("recipe-a"));
        }
        if operation == "ListRulesets" {
            first.insert(
                "TargetArn".to_owned(),
                json!("arn:aws:databrew:us-east-1:123456789012:dataset/source"),
            );
        }
        let mut second = first.clone();
        second.insert("Name".to_owned(), json!("b"));
        second.insert(
            "ResourceArn".to_owned(),
            json!(format!(
                "arn:aws:databrew:us-east-1:123456789012:{resource}/b"
            )),
        );
        let body = serde_json::to_vec(&json!({member:[first, second]}))?;
        let plan = plan(&catalog, "databrew", operation)?;
        let saved = persist(
            catalog.raw_bytes(),
            plan,
            observations(plan, rows(plan, &body)?)?,
        )
        .await?;
        let ids = saved
            .iter()
            .map(|(_, id, _)| id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["a", "b"]);
    }
    Ok(())
}

#[test]
fn given_datazone_environment_without_id_when_identity_is_extracted_then_failure_is_explicit_not_fabricated()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "datazone", "ListEnvironments")?;
    let body = br#"{"items":[{"domainId":"dzd_12345678901234","projectId":"project-a","name":"environment-a","createdBy":"user-a","provider":"Amazon Web Services"}]}"#;
    let rows = rows(plan, body)?;
    let row = rows.first().ok_or("environment row absent")?;
    assert!(matches!(
        resource_id(row, plan.identity(), None),
        Err(SweepError::Operation("modeled identity path is absent"))
    ));
    Ok(())
}

#[tokio::test]
async fn given_iot_security_profile_pages_when_wire_cursor_is_replayed_then_every_real_arn_is_stored()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iot", "ListSecurityProfiles")?;
    let first = br#"{"securityProfileIdentifiers":[{"name":"profile-a","arn":"arn:aws:iot:us-east-1:123456789012:securityprofile/profile-a"}],"nextToken":"page-2"}"#;
    let response = decode(plan, first)?;
    let token = plan
        .pagination()
        .tokens
        .first()
        .ok_or("profile cursor absent")?;
    let value = path_value(response.root(), &token.response_path)?
        .ok_or("wire cursor absent")?
        .to_value()?;
    let next = request(
        plan,
        &[input::InputField {
            path: token.request_path.clone(),
            value,
        }],
    )?;
    assert_eq!(next.uri().query(), Some("nextToken=page-2"));
    let second = br#"{"securityProfileIdentifiers":[{"name":"profile-b","arn":"arn:aws:iot:us-east-1:123456789012:securityprofile/profile-b"}]}"#;
    let mut selected = item_records(&response, plan.items())?;
    selected.extend(rows(plan, second)?);
    let saved = persist(catalog.raw_bytes(), plan, observations(plan, selected)?).await?;
    assert_eq!(
        saved
            .iter()
            .map(|(_, id, _)| id.as_str())
            .collect::<Vec<_>>(),
        [
            "arn:aws:iot:us-east-1:123456789012:securityprofile/profile-a",
            "arn:aws:iot:us-east-1:123456789012:securityprofile/profile-b",
        ]
    );
    assert!(
        saved
            .iter()
            .all(|(kind, _, _)| kind == "AWS::IoT::SecurityProfile")
    );
    Ok(())
}

#[tokio::test]
async fn given_distinct_workspaces_when_entities_are_bound_then_workspace_paths_and_entity_arns_do_not_collapse()
-> TestResult {
    let catalog = catalog()?;
    let parent = plan(&catalog, "iottwinmaker", "ListWorkspaces")?;
    let child = plan(&catalog, "iottwinmaker", "ListEntities")?;
    let mut selected = Vec::new();
    for workspace in ["workspace-a", "workspace-b"] {
        let body = serde_json::to_vec(&json!({"workspaceSummaries":[{"workspaceId":workspace,
            "arn":format!("arn:aws:iottwinmaker:us-east-1:123456789012:workspace/{workspace}"),"creationDateTime":0,"updateDateTime":0}]}))?;
        let parents = rows(parent, &body)?;
        let fields = parent_fields(child, parent, parents.first().ok_or("workspace absent")?)?;
        let request = request(child, &fields)?;
        assert_eq!(
            request.uri().path(),
            format!("/workspaces/{workspace}/entities-list")
        );
        let response = serde_json::to_vec(
            &json!({"entitySummaries":[{"entityId":"same-local-id","entityName":"sensor",
            "arn":format!("arn:aws:iottwinmaker:us-east-1:123456789012:workspace/{workspace}/entity/same-local-id"),
            "status":{"state":"ACTIVE"},"creationDateTime":0,"updateDateTime":0}]}),
        )?;
        selected.extend(rows(child, &response)?);
    }
    let observed = observations(child, selected)?;
    assert_eq!(observed.len(), 2);
    assert_ne!(
        observed.first().ok_or("first entity absent")?.key(),
        observed.last().ok_or("last entity absent")?.key()
    );
    assert!(
        observed
            .iter()
            .all(|observation| observation.key().kind.as_str() == "AWS::IoTTwinMaker::Entity")
    );
    Ok(())
}

#[test]
fn given_greengrass_history_when_real_request_is_encoded_then_superseded_deployments_are_requested()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "greengrassv2", "ListDeployments")?;
    let request = request(plan, &[])?;
    assert_eq!(request.uri().query(), Some("historyFilter=ALL"));
    let selected = rows(
        plan,
        br#"{"deployments":[{"deploymentId":"old"},{"deploymentId":"current"}]}"#,
    )?;
    let ids = selected
        .iter()
        .map(|row| resource_id(row, plan.identity(), None))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        ids.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        ["old", "current"]
    );
    Ok(())
}

#[test]
fn given_global_accelerator_called_from_another_home_region_then_control_plane_is_fixed_but_identity_remains_global()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "global-accelerator", "ListAccelerators")?;
    for home in ["us-east-1", "eu-west-1"] {
        let region = crate::execution::routing::request_region(
            plan,
            &Region::Global,
            &RegionId::parse(home)?,
            None,
        )?;
        let endpoint = crate::execution::routing::resolve(plan, &region)?;
        assert_eq!(
            endpoint.url,
            "https://globalaccelerator.us-west-2.amazonaws.com"
        );
        assert_eq!(endpoint.signing_region, "us-west-2");
    }
    let body = br#"{"Accelerators":[{"AcceleratorArn":"arn:aws:globalaccelerator::123456789012:accelerator/abc"}]}"#;
    let observed = observations(plan, rows(plan, body)?)?;
    let key = observed.first().ok_or("accelerator absent")?.key();
    assert_eq!(key.region, Region::Global);
    assert_eq!(
        key.id.as_str(),
        "arn:aws:globalaccelerator::123456789012:accelerator/abc"
    );
    Ok(())
}

#[test]
fn given_docdb_root_when_query_is_encoded_then_engine_filter_is_a_real_structured_wire_input()
-> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "docdb", "DescribeDBClusters")?;
    let request = request(plan, &[])?;
    let body = std::str::from_utf8(request.body())?;
    assert!(
        body.split('&')
            .any(|pair| pair == "Filters.Filter.1.Name=engine")
    );
    assert!(
        body.split('&')
            .any(|pair| pair == "Filters.Filter.1.Values.Value.1=docdb")
    );
    Ok(())
}
