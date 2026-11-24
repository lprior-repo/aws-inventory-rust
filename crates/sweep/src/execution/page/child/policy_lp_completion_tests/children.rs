use super::*;
mod definitions;
use definitions::{cases, child_row};
use inventory_core::catalog::runtime::CallablePlan;

struct Case {
    service: &'static str,
    version: &'static str,
    root: &'static str,
    child: &'static str,
    root_kind: &'static str,
    kind: &'static str,
    roots: Json,
    parents: [&'static str; 2],
    items: &'static str,
    identity: &'static str,
    relation: &'static str,
    paginated: bool,
}

#[tokio::test]
async fn given_two_owned_parents_when_native_child_pages_resume_then_every_child_and_exact_parent_edge_persists()
-> TestResult {
    for case in cases() {
        let bytes = compiled(
            case.service,
            case.version,
            &[case.root, case.child],
            case.root_kind,
        )?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let scenario = Scenario::new(&catalog, case.child).await?;
        scenario
            .seed(plan(&catalog, case.service, case.root)?)
            .await?;
        let root_lease = scenario.claim().await?;
        assert_eq!(root_lease.scope().operation().as_str(), case.root);
        let root = plan(&catalog, case.service, case.root)?;
        let (request, wire) = scenario.request(root, &root_lease)?;
        if case.child == "ListWorkflowVersions" {
            assert!(native_input(&wire)?.contains("type=PRIVATE"));
        }
        scenario
            .apply(&catalog, root, root_lease, &request, case.roots.clone())
            .await?;
        let child = plan(&catalog, case.service, case.child)?;
        let expected = child_pages(&case, &scenario, &catalog, child).await?;
        assert_eq!(
            scenario.stored(case.kind).await?,
            expected,
            "{}",
            case.child
        );
    }
    Ok(())
}

type ExpectedGraph = (BTreeSet<String>, BTreeSet<super::fixture::Edge>);

async fn child_pages(
    case: &Case,
    scenario: &Scenario<'_, '_>,
    catalog: &RuntimeCatalog<'_>,
    child: &CallablePlan<'_>,
) -> TestResult<ExpectedGraph> {
    let mut expected = ExpectedGraph::default();
    let mut seen = BTreeSet::new();
    for _ in 0..if case.paginated { 4 } else { 2 } {
        let lease = scenario.claim().await?;
        assert_eq!(lease.scope().operation().as_str(), case.child);
        let second = lease.token().is_some();
        let parent_key = lease
            .scope()
            .parent()
            .ok_or("child lost discovered owner")?;
        let owner = parent_key.id.as_str();
        let (request, wire) = scenario.request(child, &lease)?;
        let parent = bound(child, &request)?;
        assert!(case.parents.contains(&parent.as_str()));
        assert_parent(&wire, &parent)?;
        assert_cursor(&wire, second)?;
        assert!(seen.insert((parent.clone(), second)));
        let rows = if case.paginated {
            vec![child_row(case, &parent, second)?]
        } else {
            vec![
                child_row(case, &parent, false)?,
                child_row(case, &parent, true)?,
            ]
        };
        expect_child_rows(case, &rows, &parent, owner, &mut expected)?;
        let body = json!({(case.items):rows});
        let token = if case.service == "pca-connector-ad" {
            "NextToken"
        } else {
            "nextToken"
        };
        let body = if case.paginated {
            page(body, token, second)?
        } else {
            body
        };
        scenario
            .apply(catalog, child, lease, &request, body)
            .await?;
    }
    assert_eq!(
        seen.len(),
        if case.paginated { 4 } else { 2 },
        "{}",
        case.child
    );
    Ok(expected)
}

fn expect_child_rows(
    case: &Case,
    rows: &[Json],
    parent: &str,
    owner: &str,
    expected: &mut ExpectedGraph,
) -> TestResult {
    for row in rows {
        let value = row
            .get(case.identity)
            .and_then(Json::as_str)
            .ok_or("fixture native child identity absent")?;
        let id = if matches!(
            case.child,
            "ListBotAliases"
                | "ListBotVersions"
                | "ListPrivateGraphEndpoints"
                | "ListServicePrincipalNames"
        ) {
            format!("{}:{}|{}:{}", parent.len(), parent, value.len(), value)
        } else {
            value.to_owned()
        };
        assert!(
            expected.0.insert(id.clone()),
            "different pages must retain distinct identities: {id}"
        );
        if case.child == "ListEventRules" {
            expected
                .1
                .insert((id.clone(), id.clone(), "REFERENCES".to_owned()));
        }
        expected
            .1
            .insert((id, owner.to_owned(), case.relation.to_owned()));
    }
    Ok(())
}

#[tokio::test]
async fn given_pca_templates_with_shared_group_sid_when_nested_acl_pages_resume_then_template_scoped_entries_do_not_collapse()
-> TestResult {
    let bytes = compiled(
        "pca-connector-ad",
        "2018-05-10",
        &[
            "ListConnectors",
            "ListTemplates",
            "ListTemplateGroupAccessControlEntries",
        ],
        "AWS::PCAConnectorAD::Connector",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "pca-template-acl").await?;
    scenario
        .seed(plan(&catalog, "pca-connector-ad", "ListConnectors")?)
        .await?;
    let mut expected = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for _ in 0..7 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let plan = plan(&catalog, "pca-connector-ad", &operation)?;
        let (request, wire) = scenario.request(plan, &lease)?;
        let second = lease.token().is_some();
        assert_cursor(&wire, second)?;
        let body = match operation.as_str() {
            "ListConnectors" => {
                json!({"Connectors":[{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:connector/11111111-1111-1111-1111-111111111111"}]})
            }
            "ListTemplates" => {
                let parent = bound(plan, &request)?;
                assert_parent(&wire, &parent)?;
                page(
                    json!({"Templates":[{"Arn":format!("{parent}/template/{}", if second { "22222222-2222-2222-2222-222222222222" } else { "11111111-1111-1111-1111-111111111111" })}]}),
                    "NextToken",
                    second,
                )?
            }
            "ListTemplateGroupAccessControlEntries" => {
                let parent = bound(plan, &request)?;
                assert_parent(&wire, &parent)?;
                let sid = if second { "S-1-5-21-2" } else { "S-1-5-21-1" };
                let id = format!("{}:{}|{}:{}", parent.len(), parent, sid.len(), sid);
                expected.insert(id.clone());
                edges.insert((id, parent.clone(), "IN_TEMPLATE".to_owned()));
                page(
                    json!({"AccessControlEntries":[{"GroupSecurityIdentifier":sid}]}),
                    "NextToken",
                    second,
                )?
            }
            _ => return Err("unexpected content or write operation scheduled".into()),
        };
        scenario
            .apply(&catalog, plan, lease, &request, body)
            .await?;
    }
    assert_eq!(expected.len(), 4);
    assert_eq!(
        scenario
            .stored("AWS::PCAConnectorAD::TemplateGroupAccessControlEntry")
            .await?,
        (expected, edges)
    );
    Ok(())
}
