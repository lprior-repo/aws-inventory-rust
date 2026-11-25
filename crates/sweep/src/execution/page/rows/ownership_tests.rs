use super::*;
use crate::record::policy_dfgk_tests::fixture;
use inventory_core::{
    inventory::{AccountId, Kind},
    record::Value,
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Scope {
    account: AccountId,
    home: RegionId,
    request: Record,
}

impl Scope {
    fn new() -> TestResult<Self> {
        Ok(Self {
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-east-1")?,
            request: Record::encode(&Value::Map(BTreeMap::new()))?,
        })
    }

    fn identity(&self) -> record::IdentityContext<'_> {
        record::IdentityContext {
            account: &self.account,
            region: &self.home,
        }
    }

    fn legacy(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        rows: &[Record],
    ) -> Result<Vec<Observation>, SweepError> {
        rows.iter()
            .map(|row| {
                observation(
                    catalog,
                    self.identity(),
                    plan,
                    &Region::Regional(self.home.clone()),
                    row.clone(),
                    &self.request,
                    &DiscoveryContext::empty(),
                )
            })
            .collect()
    }

    fn prepare(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        rows: &[Record],
    ) -> Result<Vec<RowIdentity>, SweepError> {
        identities(
            catalog,
            self.identity(),
            plan,
            &Region::Regional(self.home.clone()),
            rows,
            &self.request,
            &vec![DiscoveryContext::empty(); rows.len()],
        )
    }

    fn references(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        rows: &[Record],
        identities: &[impl AsRef<ResourceKey>],
    ) -> Result<Vec<UnresolvedReference>, SweepError> {
        references(
            catalog,
            plan,
            &self.home,
            rows,
            identities,
            &self.request,
            None,
        )
    }
}

fn page_rows(plan: &CallablePlan<'_>) -> TestResult<Vec<Record>> {
    let body = serde_json::json!({"workspaceSummaries":[
        {"workspaceId":"one","arn":"arn:aws:iottwinmaker:us-east-1:123456789012:workspace/one",
         "creationDateTime":0,"updateDateTime":1},
        {"workspaceId":"two","arn":"arn:aws:iottwinmaker:us-east-1:123456789012:workspace/two",
         "creationDateTime":2,"updateDateTime":3}
    ]});
    let response = fixture::decode_as_sweep(plan, &serde_json::to_vec(&body)?)?;
    Ok(record::item_records(&response, plan.items())?)
}

const BASELINE: [(&str, &str, &str); 2] = [
    (
        "arn:aws:iottwinmaker:us-east-1:123456789012:workspace/one",
        "6ca05d75e1b1050ef45a0b74a1aa8bd04023f63b3a9937c00f4a75ee3e7ff8b9",
        "a4197c58783961726e3a6177733a696f747477696e6d616b65723a75732d656173742d313a3132333435363738393031323a776f726b73706163652f6f6e6519870bc182000019bc24c182010019bf7d636f6e65",
    ),
    (
        "arn:aws:iottwinmaker:us-east-1:123456789012:workspace/two",
        "a368c8749a0928ac5508dbe09eb495bd0a6b02658bbeede6c9930b2d71119f2b",
        "a4197c58783961726e3a6177733a696f747477696e6d616b65723a75732d656173742d313a3132333435363738393031323a776f726b73706163652f74776f19870bc182020019bc24c182030019bf7d6374776f",
    ),
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn given_fixture_page_when_rows_move_into_observations_then_pre_change_bytes_and_hashes_are_exact()
-> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iottwinmaker", "ListWorkspaces")?;
    let rows = page_rows(plan)?;
    assert_eq!(rows.len(), 2);
    let pointers = rows
        .iter()
        .map(|row| row.as_bytes().as_ptr())
        .collect::<Vec<_>>();
    let scope = Scope::new()?;
    let prepared = scope.prepare(&catalog, plan, &rows)?;
    let moved = observations(
        rows,
        prepared,
        plan.hash_exclusions(),
        plan.facet_precedence(),
    )?;
    assert_eq!(moved.len(), 2);
    for ((actual, (id, hash, bytes)), pointer) in moved.iter().zip(BASELINE).zip(pointers) {
        let expected = ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::Regional(RegionId::parse("us-east-1")?),
            kind: Kind::parse("AWS::IoTTwinMaker::Workspace")?,
            id: ResourceId::parse(id)?,
        };
        assert_eq!(actual.key(), &expected);
        assert_eq!(hex(actual.record().as_bytes()), bytes);
        assert_eq!(hex(actual.content_hash().as_bytes()), hash);
        assert_eq!(actual.hash_exclusions(), plan.hash_exclusions());
        assert_eq!(actual.facet_precedence(), plan.facet_precedence());
        assert_eq!(actual.record().as_bytes().as_ptr(), pointer);
    }
    Ok(())
}

#[test]
fn given_mismatched_owned_rows_when_assembling_observations_then_page_is_rejected() -> TestResult {
    let row = Record::encode(&Value::Map(BTreeMap::new()))?;
    let error = observations(
        vec![row],
        Vec::new(),
        &inventory_core::record::Exclusions::none(),
        0,
    )
    .err()
    .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "sweep operation failed: rows and observation identities exceed bounds or have different lengths"
    );
    Ok(())
}

#[test]
fn given_too_many_owned_rows_when_assembling_observations_then_page_is_rejected() -> TestResult {
    let row = Record::encode(&Value::Map(BTreeMap::new()))?;
    let key = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::Global,
        kind: inventory_core::inventory::Kind::parse("AWS::IAM::Role")?,
        id: inventory_core::inventory::ResourceId::parse("role")?,
    };
    let count = MAX_PAGE_ROWS.saturating_add(1);
    let identities = (0..count)
        .map(|_| RowIdentity {
            key: key.clone(),
            name: None,
        })
        .collect();
    let error = observations(
        vec![row; count],
        identities,
        &inventory_core::record::Exclusions::none(),
        0,
    )
    .err()
    .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "sweep operation failed: rows and observation identities exceed bounds or have different lengths"
    );
    Ok(())
}

#[test]
fn given_exactly_max_page_rows_when_assembling_observations_then_page_is_accepted() -> TestResult {
    let row = Record::encode(&Value::Map(BTreeMap::new()))?;
    let key = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::Global,
        kind: inventory_core::inventory::Kind::parse("AWS::IAM::Role")?,
        id: inventory_core::inventory::ResourceId::parse("role")?,
    };
    let identities = (0..MAX_PAGE_ROWS)
        .map(|_| RowIdentity {
            key: key.clone(),
            name: None,
        })
        .collect();
    let observations = observations(
        vec![row; MAX_PAGE_ROWS],
        identities,
        &inventory_core::record::Exclusions::none(),
        0,
    )?;
    assert_eq!(observations.len(), MAX_PAGE_ROWS);
    Ok(())
}

fn assert_references_match(actual: &[UnresolvedReference], expected: &[UnresolvedReference]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.from, expected.from);
        assert_eq!(actual.relation, expected.relation);
        assert_eq!(actual.source, expected.source);
        assert_eq!(actual.target.account(), expected.target.account());
        assert_eq!(actual.target.region(), expected.target.region());
        assert_eq!(actual.target.kind(), expected.target.kind());
        assert_eq!(
            actual.target.candidate_kinds(),
            expected.target.candidate_kinds()
        );
        assert_eq!(actual.target.path(), expected.target.path());
        assert_eq!(actual.target.value(), expected.target.value());
    }
}

#[test]
fn given_scene_page_and_workspace_request_when_prepared_then_nonempty_references_match_legacy()
-> TestResult {
    let bytes = crate::execution::test_catalog::compile(
        "iottwinmaker",
        "2021-11-29",
        &["ListWorkspaces", "ListScenes"],
        None,
        |_, _| Ok(()),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = fixture::plan(&catalog, "iottwinmaker", "ListScenes")?;
    let body = serde_json::json!({"sceneSummaries":[
        {"sceneId":"one","arn":"arn:aws:iottwinmaker:us-east-1:123456789012:workspace/factory/scene/one",
         "creationDateTime":0,"updateDateTime":1},
        {"sceneId":"two","arn":"arn:aws:iottwinmaker:us-east-1:123456789012:workspace/factory/scene/two",
         "creationDateTime":2,"updateDateTime":3}
    ]});
    let response = fixture::decode_as_sweep(plan, &serde_json::to_vec(&body)?)?;
    let rows = record::item_records(&response, plan.items())?;
    let kind = plan.kind();
    let relation = catalog
        .relationships_from(kind)
        .find(|relation| relation.relation.as_str() == "IN_WORKSPACE")
        .ok_or("scene workspace relation absent")?;
    let field = relation
        .source_path
        .first()
        .ok_or("workspace field absent")?;
    assert_eq!(catalog.catalog().field_name(*field)?, "workspaceId");
    let mut scope = Scope::new()?;
    scope.request = Record::encode(&Value::Map(BTreeMap::from([(
        inventory_core::record::Key::Field(*field),
        Value::Text("factory".to_owned()),
    )])))?;
    let legacy = scope.legacy(&catalog, plan, &rows)?;
    let prepared = scope.prepare(&catalog, plan, &rows)?;
    let expected = scope.references(&catalog, plan, &rows, &legacy)?;
    assert!(!expected.is_empty());
    assert_eq!(expected.len(), 2);
    for reference in &expected {
        assert_eq!(reference.relation.as_str(), "IN_WORKSPACE");
        assert_eq!(
            reference.target.kind().as_str(),
            "AWS::IoTTwinMaker::Workspace"
        );
        assert_eq!(reference.target.value().as_str(), "factory");
    }
    let actual = scope.references(&catalog, plan, &rows, &prepared)?;
    assert_references_match(&actual, &expected);
    Ok(())
}

#[test]
fn given_request_correlated_identity_when_request_changes_then_prepared_keys_change() -> TestResult
{
    use inventory_core::catalog::runtime::{IdentityFieldPlan, IdentityPlan};
    let bytes = crate::execution::test_catalog::compile(
        "cognito-identity-provider",
        "2016-04-18",
        &["ListUserPools", "ListIdentityProviders"],
        None,
        |_, _| Ok(()),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = fixture::plan(
        &catalog,
        "cognito-identity-provider",
        "ListIdentityProviders",
    )?;
    let IdentityPlan::CorrelatedKey(fields) = plan.identity() else {
        return Err("expected request-correlated provider identity".into());
    };
    let Some(IdentityFieldPlan::Request(path)) = fields.first() else {
        return Err("expected first identity field to read the request".into());
    };
    let field = *path.first().ok_or("request identity path absent")?;
    assert_eq!(catalog.catalog().field_name(field)?, "UserPoolId");
    let response = fixture::decode_as_sweep(
        plan,
        br#"{"Providers":[{"ProviderName":"provider","ProviderType":"SAML"}]}"#,
    )?;
    let rows = record::item_records(&response, plan.items())?;
    assert_eq!(rows.len(), 1);
    let mut scope = Scope::new()?;
    scope.home = RegionId::parse("us-west-2")?;
    for (pool, expected) in [
        ("us-west-2_A", "11:us-west-2_A|8:provider"),
        ("us-west-2_B", "11:us-west-2_B|8:provider"),
    ] {
        scope.request = Record::encode(&Value::Map(BTreeMap::from([(
            inventory_core::record::Key::Field(field),
            Value::Text(pool.to_owned()),
        )])))?;
        let prepared = scope.prepare(&catalog, plan, &rows)?;
        assert_eq!(prepared.len(), 1);
        assert_eq!(
            prepared.first().ok_or("prepared provider absent")?.as_ref(),
            &ResourceKey {
                account: AccountId::parse("123456789012")?,
                region: Region::Regional(RegionId::parse("us-west-2")?),
                kind: Kind::parse("AWS::Cognito::UserPoolIdentityProvider")?,
                id: ResourceId::parse(expected)?,
            }
        );
    }
    Ok(())
}
