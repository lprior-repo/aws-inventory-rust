use super::super::persistence::key;
use super::*;
use inventory_engine::inventory::WorkLease;
const ORGANIZATION: &str = "AWS::ApiOnly::WorkMailOrganization";
const ENTITIES: [(&str, &str, &str, &str); 3] = [
    ("ListUsers", "Users", "user", "AWS::ApiOnly::WorkMailUser"),
    (
        "ListGroups",
        "Groups",
        "group",
        "AWS::ApiOnly::WorkMailGroup",
    ),
    (
        "ListResources",
        "Resources",
        "resource",
        "AWS::ApiOnly::WorkMailResource",
    ),
];
const CHILDREN: [(&str, &str, &str, &str); 3] = [
    (
        "ListAliases",
        "Aliases",
        "alias@example.test",
        "AWS::ApiOnly::WorkMailAlias",
    ),
    (
        "ListGroupsForEntity",
        "Groups",
        "group",
        "AWS::ApiOnly::WorkMailEntityGroup",
    ),
    (
        "ListMailboxPermissions",
        "Permissions",
        "grantee",
        "AWS::ApiOnly::WorkMailMailboxPermission",
    ),
];
fn allowed(operation: &str, entity: &str) -> bool {
    match operation {
        "ListAliases" => true,
        "ListGroupsForEntity" => entity != "resource",
        "ListMailboxPermissions" => entity != "group",
        _ => false,
    }
}
fn text<'a>(json: &'a Json, name: &str) -> TestResult<&'a str> {
    json.get(name)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("native mail request omitted {name}").into())
}
fn page_body(items: &str, row: Json, first: bool) -> Json {
    if first {
        json!({items:[],"NextToken":"native-next"})
    } else {
        json!({items:[row]})
    }
}

pub(super) fn page(
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
    wire: &http::Request<bytes::Bytes>,
    reverse: bool,
) -> TestResult<Json> {
    let request: Json = serde_json::from_slice(wire.body())?;
    let first = lease.token().is_none();
    assert_eq!(
        request.get("NextToken").and_then(Json::as_str),
        if first { None } else { Some("native-next") }
    );
    let operation = plan.operation_name();
    if operation == "ListOrganizations" {
        assert!(request.get("OrganizationId").is_none());
        assert!(request.get("EntityId").is_none());
        if first {
            return Ok(json!({"OrganizationSummaries":[],"NextToken":"native-next"}));
        }
        let mut rows = vec![
            json!({"OrganizationId":"org-a","Alias":"a"}),
            json!({"OrganizationId":"org-b","Alias":"b"}),
        ];
        if reverse {
            rows.reverse();
        }
        return Ok(json!({"OrganizationSummaries":rows}));
    }
    assert!(["org-a", "org-b"].contains(&text(&request, "OrganizationId")?));
    if let Some((_, items, entity, _)) = ENTITIES.iter().find(|(name, _, _, _)| *name == operation)
    {
        assert!(request.get("EntityId").is_none());
        return Ok(page_body(items, json!({"Id":entity,"Name":entity}), first));
    }
    let entity = text(&request, "EntityId")?;
    assert!(["user", "group", "resource"].contains(&entity));
    assert!(
        allowed(operation, entity),
        "forbidden WorkMail parent/consumer pairing"
    );
    let row = match operation {
        "ListAliases" => json!("alias@example.test"),
        "ListGroupsForEntity" => json!({"GroupId":"group","GroupName":"group"}),
        "ListMailboxPermissions" => {
            json!({"GranteeId":"grantee","GranteeType":"USER","PermissionValues":["FULL_ACCESS"]})
        }
        _ => return Err("unexpected WorkMail detail operation".into()),
    };
    let (_, items, _, _) = CHILDREN
        .iter()
        .find(|(name, _, _, _)| *name == operation)
        .ok_or("unknown child")?;
    Ok(page_body(items, row, first))
}

fn composite(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| format!("{}:{part}", part.len()))
        .collect::<Vec<_>>()
        .join("|")
}

pub(super) fn expected() -> (BTreeSet<Resource>, BTreeSet<Edge>) {
    let mut resources = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for organization in ["org-a", "org-b"] {
        let org = key("us-east-1", ORGANIZATION, organization);
        resources.insert(org.clone());
        for (_, _, entity, kind) in ENTITIES {
            let owner = key("us-east-1", kind, &composite(&[organization, entity]));
            resources.insert(owner.clone());
            edges.insert((owner.clone(), "IN_ORGANIZATION".to_owned(), org.clone()));
            for (operation, _, selected, kind) in CHILDREN {
                if !allowed(operation, entity) {
                    continue;
                }
                let child = key(
                    "us-east-1",
                    kind,
                    &composite(&[organization, entity, selected]),
                );
                resources.insert(child.clone());
                edges.insert((child.clone(), "IN_ORGANIZATION".to_owned(), org.clone()));
                edges.insert((child.clone(), "FOR_ENTITY".to_owned(), owner.clone()));
                if operation == "ListGroupsForEntity" {
                    edges.insert((
                        child,
                        "IN_GROUP".to_owned(),
                        key(
                            "us-east-1",
                            "AWS::ApiOnly::WorkMailGroup",
                            &composite(&[organization, "group"]),
                        ),
                    ));
                }
            }
        }
    }
    (resources, edges)
}
