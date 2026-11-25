use super::*;
mod transport;

const ROOTS: &[(&str, &str)] = &[
    ("ListWebACLs", "GetWebACL"),
    ("ListIPSets", "GetIPSet"),
    ("ListRegexPatternSets", "GetRegexPatternSet"),
    ("ListRuleGroups", "GetRuleGroup"),
    ("ListManagedRuleSets", "GetManagedRuleSet"),
    ("ListLoggingConfigurations", "GetLoggingConfiguration"),
];

#[tokio::test]
async fn given_wafv2_scope_branches_when_sweeping_then_only_matching_child_requests_persist()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let mut scenario = Scenario::new(&catalog, "wafv2-branch-provenance").await?;
    scenario.home = RegionId::parse("eu-west-1")?;
    seed_roots(&scenario, &catalog).await?;
    let mut transport = transport::FakeTransport::default();
    let (scenario, seen) = scenario
        .drain(&catalog, "wafv2", |plan, _, wire, resumed| {
            assert!(!resumed);
            transport.respond(plan, wire)
        })
        .await?;
    assert_requests(&transport, &seen);
    assert_durable_web_acls(&scenario).await?;
    Ok(())
}

async fn seed_roots(scenario: &Scenario<'_, '_>, catalog: &RuntimeCatalog<'_>) -> TestResult {
    for (operation, _) in ROOTS {
        let root = catalog
            .lookup("wafv2", operation)
            .ok_or("missing WAFv2 root")?;
        for branch in root.branches() {
            let work = root_work(scenario, root, branch)?;
            scenario.store.seed_work(work).await?;
        }
    }
    Ok(())
}

fn root_work(
    scenario: &Scenario<'_, '_>,
    root: &CallablePlan<'_>,
    branch: &inventory_core::catalog::runtime::ExecutionBranch,
) -> TestResult<WorkSpec> {
    let region = branch_region(scenario, branch.scope);
    let scope = ScopeKey::new(
        scenario.account.clone(),
        region.clone(),
        ServiceId::parse("wafv2")?,
        OperationId::parse(root.operation_name())?,
        None,
    )?
    .with_branch(branch.discriminator.clone())?;
    let bindings = routing::branch_bindings(root, branch, &scenario.account, &region)?;
    Ok(WorkSpec::new(
        scenario.sweep.clone(),
        scope,
        None,
        bindings.into_bytes(),
    )?)
}

fn branch_region(scenario: &Scenario<'_, '_>, scope: PlanScope) -> Region {
    if scope == PlanScope::Account {
        Region::Global
    } else {
        Region::Regional(scenario.home.clone())
    }
}

fn assert_requests(transport: &transport::FakeTransport, seen: &BTreeMap<String, usize>) {
    assert_eq!(seen.len(), 12);
    assert_eq!(transport.seen.len(), 24);
    for (root, child) in ROOTS {
        for operation in [root, child] {
            assert_eq!(seen.get(*operation), Some(&2));
            for scope in ["REGIONAL", "CLOUDFRONT"] {
                assert_eq!(
                    transport
                        .seen
                        .get(&(operation.to_string(), scope.to_owned())),
                    Some(&1)
                );
            }
        }
    }
}

async fn assert_durable_web_acls(scenario: &Scenario<'_, '_>) -> TestResult {
    let reader = scenario.reader().await?;
    let actual: Vec<(String, String)> = sqlx::query_as(
        "SELECT region,id FROM resources WHERE kind='AWS::WAFv2::WebACL' AND account='123456789012' ORDER BY region,id",
    ).fetch_all(&reader).await?;
    reader.close().await;
    let expected = [
        ("eu-west-1".to_owned(), transport::arn("REGIONAL", "webacl")),
        ("global".to_owned(), transport::arn("CLOUDFRONT", "webacl")),
    ];
    assert_eq!(actual, expected);
    Ok(())
}
