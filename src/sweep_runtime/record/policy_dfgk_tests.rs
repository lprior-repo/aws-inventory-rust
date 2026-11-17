pub(in crate::sweep_runtime) mod fixture;
mod principals;
mod final_sitewise;

use super::*;
use fixture::{catalog, decode, observations, parent_fields, persist, plan, request, rows};
use inventory_core::{catalog::runtime::{CallablePlan, RuntimeCatalog}, inventory::Region};
use crate::sweep_runtime::input;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_mixed_search_engines_when_real_requests_are_encoded_then_domains_have_disjoint_canonical_nodes() -> TestResult {
    // Given both production API policies and the actual pinned Smithy wire models.
    let catalog = catalog()?;
    let mut keys = std::collections::BTreeSet::new();
    for (service, engine, name, kind) in [
        ("elasticsearch-service", "Elasticsearch", "legacy-a", "AWS::Elasticsearch::Domain"),
        ("opensearch", "OpenSearch", "modern-a", "AWS::OpenSearchService::Domain"),
    ] {
        let plan = plan(&catalog, service, "ListDomainNames")?;
        // When the request goes through the production numeric binding/REST encoder.
        let request = request(plan, &[])?;
        assert_eq!(request.uri().query(), Some(format!("engineType={engine}").as_str()));
        // Then each engine response persists exactly its own physical domain and CFN kind.
        let body = serde_json::to_vec(&json!({"DomainNames":[{"DomainName":name,"EngineType":engine}]}))?;
        let saved = persist(catalog.raw_bytes(), plan, observations(plan, rows(plan, &body)?)?).await?;
        assert_eq!(saved.len(), 1);
        let (saved_kind, id, _) = saved.first().ok_or("domain was not persisted")?;
        assert_eq!((saved_kind.as_str(), id.as_str()), (kind, name));
        keys.insert((saved_kind.clone(), id.clone()));
    }
    assert_eq!(keys.len(), 2);
    Ok(())
}

#[tokio::test]
async fn given_databrew_rows_without_optional_arns_when_collected_then_required_names_preserve_every_sibling() -> TestResult {
    let catalog = catalog()?;
    for (operation, member, resource) in [("ListJobs", "Jobs", "job"), ("ListProjects", "Projects", "project"),
        ("ListRulesets", "Rulesets", "ruleset"), ("ListSchedules", "Schedules", "schedule")] {
        // Given a model-valid row without ResourceArn, and a complete sibling.
        let mut first = serde_json::Map::from_iter([("Name".to_owned(), json!("a"))]);
        if operation == "ListProjects" { first.insert("RecipeName".to_owned(), json!("recipe-a")); }
        if operation == "ListRulesets" { first.insert("TargetArn".to_owned(), json!("arn:aws:databrew:us-east-1:123456789012:dataset/source")); }
        let mut second = first.clone();
        second.insert("Name".to_owned(), json!("b"));
        second.insert("ResourceArn".to_owned(), json!(format!("arn:aws:databrew:us-east-1:123456789012:{resource}/b")));
        let body = serde_json::to_vec(&json!({member:[first, second]}))?;
        let plan = plan(&catalog, "databrew", operation)?;
        // When the actual decoder, identity extractor and SQLite commit process both rows.
        let saved = persist(catalog.raw_bytes(), plan, observations(plan, rows(plan, &body)?)?).await?;
        // Then neither a missing optional ARN nor a different sibling ARN changes the Name key.
        let ids = saved.iter().map(|(_, id, _)| id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, ["a", "b"]);
    }
    Ok(())
}

#[test]
fn given_datazone_environment_without_id_when_identity_is_extracted_then_failure_is_explicit_not_fabricated() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "datazone", "ListEnvironments")?;
    // Required metadata does not prove a unique environment identity if optional id is absent.
    let body = br#"{"items":[{"domainId":"dzd_12345678901234","projectId":"project-a","name":"environment-a","createdBy":"user-a","provider":"Amazon Web Services"}]}"#;
    let rows = rows(plan, body)?;
    let row = rows.first().ok_or("environment row absent")?;
    assert!(matches!(resource_id(row, plan.identity(), None),
        Err(SweepError::Operation("modeled identity path is absent"))));
    Ok(())
}

#[tokio::test]
async fn given_iot_security_profile_pages_when_wire_cursor_is_replayed_then_every_real_arn_is_stored() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iot", "ListSecurityProfiles")?;
    let first = br#"{"securityProfileIdentifiers":[{"name":"profile-a","arn":"arn:aws:iot:us-east-1:123456789012:securityprofile/profile-a"}],"nextToken":"page-2"}"#;
    let response = decode(plan, first)?;
    let token = plan.pagination().tokens.first().ok_or("profile cursor absent")?;
    let value = path_value(response.root(), &token.response_path)?.ok_or("wire cursor absent")?.to_value()?;
    // The compiled token paths, not assumed field names, drive the next real request.
    let next = request(plan, &[input::InputField { path: token.request_path.clone(), value }])?;
    assert_eq!(next.uri().query(), Some("nextToken=page-2"));
    let second = br#"{"securityProfileIdentifiers":[{"name":"profile-b","arn":"arn:aws:iot:us-east-1:123456789012:securityprofile/profile-b"}]}"#;
    let mut selected = item_records(&response, plan.items())?;
    selected.extend(rows(plan, second)?);
    let saved = persist(catalog.raw_bytes(), plan, observations(plan, selected)?).await?;
    assert_eq!(saved.iter().map(|(_, id, _)| id.as_str()).collect::<Vec<_>>(), [
        "arn:aws:iot:us-east-1:123456789012:securityprofile/profile-a",
        "arn:aws:iot:us-east-1:123456789012:securityprofile/profile-b",
    ]);
    assert!(saved.iter().all(|(kind, _, _)| kind == "AWS::IoT::SecurityProfile"));
    Ok(())
}

#[tokio::test]
async fn given_distinct_workspaces_when_entities_are_bound_then_workspace_paths_and_entity_arns_do_not_collapse() -> TestResult {
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
        assert_eq!(request.uri().path(), format!("/workspaces/{workspace}/entities-list"));
        let response = serde_json::to_vec(&json!({"entitySummaries":[{"entityId":"same-local-id","entityName":"sensor",
            "arn":format!("arn:aws:iottwinmaker:us-east-1:123456789012:workspace/{workspace}/entity/same-local-id"),
            "status":{"state":"ACTIVE"},"creationDateTime":0,"updateDateTime":0}]}))?;
        selected.extend(rows(child, &response)?);
    }
    let observed = observations(child, selected)?;
    assert_eq!(observed.len(), 2);
    assert_ne!(observed.first().ok_or("first entity absent")?.key(), observed.last().ok_or("last entity absent")?.key());
    assert!(observed.iter().all(|observation| observation.key().kind.as_str() == "AWS::IoTTwinMaker::Entity"));
    Ok(())
}

#[test]
fn given_greengrass_history_when_real_request_is_encoded_then_superseded_deployments_are_requested() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "greengrassv2", "ListDeployments")?;
    let request = request(plan, &[])?;
    assert_eq!(request.uri().query(), Some("historyFilter=ALL"));
    let selected = rows(plan, br#"{"deployments":[{"deploymentId":"old"},{"deploymentId":"current"}]}"#)?;
    let ids = selected.iter().map(|row| resource_id(row, plan.identity(), None))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(ids.iter().map(|id| id.as_str()).collect::<Vec<_>>(), ["old", "current"]);
    Ok(())
}

#[test]
fn given_global_accelerator_called_from_another_home_region_then_control_plane_is_fixed_but_identity_remains_global() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "global-accelerator", "ListAccelerators")?;
    for home in ["us-east-1", "eu-west-1"] {
        let region = crate::sweep_runtime::execution::routing::request_region(plan, &Region::Global, &RegionId::parse(home)?, None)?;
        let endpoint = crate::sweep_runtime::execution::routing::resolve(plan, &region)?;
        assert_eq!(endpoint.url, "https://globalaccelerator.us-west-2.amazonaws.com");
        assert_eq!(endpoint.signing_region, "us-west-2");
    }
    let body = br#"{"Accelerators":[{"AcceleratorArn":"arn:aws:globalaccelerator::123456789012:accelerator/abc"}]}"#;
    let observed = observations(plan, rows(plan, body)?)?;
    let key = observed.first().ok_or("accelerator absent")?.key();
    assert_eq!(key.region, Region::Global);
    assert_eq!(key.id.as_str(), "arn:aws:globalaccelerator::123456789012:accelerator/abc");
    Ok(())
}

#[test]
fn given_docdb_root_when_query_is_encoded_then_engine_filter_is_a_real_structured_wire_input() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "docdb", "DescribeDBClusters")?;
    let request = request(plan, &[])?;
    let body = std::str::from_utf8(request.body())?;
    assert!(body.split('&').any(|pair| pair == "Filters.Filter.1.Name=engine"));
    assert!(body.split('&').any(|pair| pair == "Filters.Filter.1.Values.Value.1=docdb"));
    Ok(())
}

#[tokio::test]
async fn given_mixed_msk_cluster_types_when_persisted_then_both_canonical_cfn_kinds_survive() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "kafka", "ListClustersV2")?;
    let body = br#"{"clusterInfoList":[{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/provisioned/1","clusterType":"PROVISIONED"},{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/serverless/2","clusterType":"SERVERLESS"}]}"#;
    let saved = persist(catalog.raw_bytes(), plan, observations(plan, rows(plan, body)?)?).await?;
    assert_eq!(saved.iter().map(|(kind, _, _)| kind.as_str()).collect::<Vec<_>>(), ["AWS::MSK::Cluster", "AWS::MSK::ServerlessCluster"]);
    let unknown = rows(plan, br#"{"clusterInfoList":[{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/unknown/3","clusterType":"FUTURE"}]}"#)?
        .into_iter().next().ok_or("unknown cluster row absent")?;
    assert!(matches!(crate::sweep_runtime::resource_projection::classify(plan, &unknown),
        Err(SweepError::Catalog(inventory_core::catalog::runtime::RuntimeCatalogError::UnclassifiedKind { discriminator })) if discriminator == "FUTURE"));
    let missing = rows(plan, br#"{"clusterInfoList":[{"clusterArn":"arn:aws:kafka:us-east-1:123456789012:cluster/missing/4"}]}"#)?
        .into_iter().next().ok_or("missing cluster row absent")?;
    assert!(matches!(crate::sweep_runtime::resource_projection::classify(plan, &missing),
        Err(SweepError::Catalog(inventory_core::catalog::runtime::RuntimeCatalogError::Capability))));
    Ok(())
}

#[tokio::test]
async fn given_iam_list_and_detail_when_projected_and_stored_then_trust_values_never_reach_sqlite() -> TestResult {
    let catalog = catalog()?;
    let document = r#"{"Statement":{"Effect":"Allow","Action":"sts:AssumeRole","Principal":{"Service":"ec2.amazonaws.com"},"Condition":{"StringEquals":{"sts:ExternalId":"must-not-persist"}}}}"#;
    let encoded = document.bytes().map(|byte| format!("%{byte:02X}")).collect::<String>();
    let role = format!("<Path>/</Path><RoleName>collector</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>arn:aws:iam::123456789012:role/collector</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>{encoded}</AssumeRolePolicyDocument>");
    let mut identities = Vec::new();
    for (operation, selected) in [("ListRoles", format!("<Roles><member>{role}</member></Roles><IsTruncated>false</IsTruncated>")),
        ("GetRole", format!("<Role>{role}</Role>"))] {
        let plan = plan(&catalog, "iam", operation)?;
        let response = format!("<{operation}Response xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><{operation}Result>{selected}</{operation}Result></{operation}Response>");
        let saved = persist(catalog.raw_bytes(), plan, observations(plan, rows(plan, response.as_bytes())?)?).await?;
        let (kind, id, record) = saved.first().ok_or("IAM role absent from SQLite")?;
        assert_eq!(kind, "AWS::IAM::Role");
        identities.push(id.clone());
        let stored = Record::from_bytes(record.clone())?;
        let projection = plan.projection().ok_or("IAM bounded projection absent")?;
        let Some(ValueRef::Map(summary)) = path_value(stored.root(), projection.protected_path())? else {
            return Err("trust summary was not stored as a structure".into());
        };
        assert!(matches!(summary.get_name("trust_policy_values_omitted")?, Some(ValueRef::Bool(true))));
        for forbidden in ["must-not-persist", document, encoded.as_str()] {
            assert!(!record.windows(forbidden.len()).any(|window| window == forbidden.as_bytes()));
        }
        for retained in ["sts:AssumeRole", "ec2.amazonaws.com", "StringEquals"] {
            assert!(record.windows(retained.len()).any(|window| window == retained.as_bytes()));
        }
    }
    assert_eq!(identities, ["arn:aws:iam::123456789012:role/collector", "arn:aws:iam::123456789012:role/collector"]);
    Ok(())
}

fn listed_role(document: &str) -> Vec<u8> {
    let encoded = document.bytes().map(|byte| format!("%{byte:02X}")).collect::<String>();
    format!("<ListRolesResponse><ListRolesResult><Roles><member><Path>/</Path><RoleName>collector</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>arn:aws:iam::123456789012:role/collector</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>{encoded}</AssumeRolePolicyDocument></member></Roles><IsTruncated>false</IsTruncated></ListRolesResult></ListRolesResponse>").into_bytes()
}

#[tokio::test]
async fn given_allow_and_deny_trust_principals_when_projected_then_only_registered_allow_arn_targets_keep_foreign_owner() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iam", "ListRoles")?;
    let document = r#"{"Statement":[{"Effect":"Allow","Action":"sts:AssumeRole","Principal":{"AWS":["arn:aws:iam::210987654321:role/external","arn:aws:iam::210987654321:root","*"],"Service":"ec2.amazonaws.com"},"Condition":{"StringEquals":{"sts:ExternalId":"private-value"}}},{"Effect":"Deny","Action":"sts:AssumeRole","Principal":{"AWS":"arn:aws:iam::210987654321:role/denied"}}]}"#;
    let observed = observations(plan, rows(plan, &listed_role(document))?)?;
    let observation = observed.first().ok_or("projected role absent")?;
    let references = crate::sweep_runtime::resource_projection::trust_references(&catalog, plan, observation)?;
    assert_eq!(references.len(), 1);
    let reference = references.first().ok_or("allow principal reference absent")?;
    assert_eq!(reference.relation.as_str(), "DECLARES_TRUST");
    assert_eq!(reference.target.account().as_str(), "210987654321");
    assert_eq!(reference.target.region(), &Region::Global);
    assert_eq!(reference.target.kind().as_str(), "AWS::IAM::Role");
    assert_eq!(reference.target.value().as_str(), "arn:aws:iam::210987654321:role/external");
    assert_eq!(reference.from.account.as_str(), "123456789012");
    let saved = persist(catalog.raw_bytes(), plan, observed).await?;
    let (_, _, record) = saved.first().ok_or("role facet absent after reference commit")?;
    assert!(!record.windows("private-value".len()).any(|window| window == b"private-value"));
    Ok(())
}

#[test]
fn given_malformed_or_overbound_iam_documents_when_projected_then_the_page_fails_closed() -> TestResult {
    let catalog = catalog()?;
    let plan = plan(&catalog, "iam", "ListRoles")?;
    for document in ["not-json".to_owned(), "x".repeat(65_537)] {
        let selected = rows(plan, &listed_role(&document))?;
        let row = selected.first().ok_or("role row absent")?;
        assert!(matches!(crate::sweep_runtime::resource_projection::project(plan, row),
            Err(SweepError::Operation("invalid bounded IAM trust document"))));
    }
    Ok(())
}

#[tokio::test]
async fn given_dms_recursive_file_body_when_catalog_is_compiled_then_that_call_is_absent_and_safe_metadata_remains_collectable() -> TestResult {
    let catalog = catalog()?;
    assert!(catalog.lookup("database-migration-service", "DescribeReplicationInstances").is_none());
    let plan = plan(&catalog, "database-migration-service", "DescribeInstanceProfiles")?;
    let body = br#"{"InstanceProfiles":[{"InstanceProfileArn":"arn:aws:dms:us-east-1:123456789012:instance-profile:safe","InstanceProfileName":"safe-profile","NetworkType":"IPV4"}]}"#;
    let saved = persist(catalog.raw_bytes(), plan, observations(plan, rows(plan, body)?)?).await?;
    assert_eq!(saved.len(), 1);
    let (kind, id, _) = saved.first().ok_or("safe DMS metadata absent")?;
    assert_eq!(kind, "AWS::DMS::InstanceProfile");
    assert_eq!(id, "arn:aws:dms:us-east-1:123456789012:instance-profile:safe");
    Ok(())
}
