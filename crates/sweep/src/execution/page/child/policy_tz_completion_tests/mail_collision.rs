use super::*;
mod shared;

#[tokio::test]
async fn given_workmail_kinds_with_the_same_local_id_when_fromany_children_execute_then_each_resolved_edge_uses_only_its_actual_native_producer()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let mut scenario = Scenario::new(&catalog, "tz-mail-colliding-parent-ids").await?;
    scenario
        .seed(&catalog, "workmail", &["ListOrganizations"])
        .await?;
    let mut actual_branches = Vec::new();
    while let Some(lease) = scenario.store.claim(&scenario.sweep, 1000, 10000).await? {
        let plan = catalog
            .lookup("workmail", lease.scope().operation().as_str())
            .ok_or("collision producer absent")?;
        let (request, wire) = scenario.request(plan, &lease)?;
        let native: Json = serde_json::from_slice(wire.body())?;
        let response = response(plan, &lease, &native, &mut actual_branches, false)?;
        scenario
            .apply(&catalog, plan, lease, &request, response)
            .await?;
        scenario = scenario.reopen(&catalog).await?;
    }
    actual_branches.sort();
    assert_eq!(
        actual_branches,
        [
            "WorkMailAlias:group",
            "WorkMailAlias:resource",
            "WorkMailAlias:user",
            "WorkMailEntityGroup:group",
            "WorkMailEntityGroup:user",
            "WorkMailMailboxPermission:resource",
            "WorkMailMailboxPermission:user"
        ]
    );
    for (kind, parents, local) in [
        ("WorkMailAlias", &["user", "group", "resource"][..], "alias"),
        (
            "WorkMailEntityGroup",
            &["user", "group"][..],
            "entity-group",
        ),
        (
            "WorkMailMailboxPermission",
            &["user", "resource"][..],
            "grantee",
        ),
    ] {
        let ids = parents
            .iter()
            .map(|parent| {
                correlated(&[
                    "m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "collision",
                    &local_id(local, parent),
                ])
            })
            .collect();
        scenario
            .assert_ids(&format!("AWS::ApiOnly::{kind}"), ids)
            .await?;
        let edges = parents
            .iter()
            .map(|parent| {
                let target = match *parent {
                    "user" => "WorkMailUser",
                    "group" => "WorkMailGroup",
                    _ => "WorkMailResource",
                };
                (
                    correlated(&[
                        "m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "collision",
                        &local_id(local, parent),
                    ]),
                    format!("AWS::ApiOnly::{target}"),
                    correlated(&["m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "collision"]),
                )
            })
            .collect();
        scenario
            .assert_edges(&format!("AWS::ApiOnly::{kind}"), "FOR_ENTITY", edges)
            .await?;
    }
    Ok(())
}

fn local_id(prefix: &str, parent: &str) -> String {
    if prefix == "alias" {
        format!("alias-for-{parent}@example.test")
    } else {
        format!("{prefix}-for-{parent}")
    }
}

fn response(
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
    native: &Json,
    branches: &mut Vec<String>,
    shared: bool,
) -> TestResult<Json> {
    let operation = plan.operation_name();
    if operation == "ListOrganizations" {
        let mut organizations = vec![
            json!({"OrganizationId":"m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","Alias":"collision","State":"Active"}),
        ];
        if shared {
            organizations.push(json!({"OrganizationId":"m-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","Alias":"second","State":"Active"}));
        }
        return Ok(json!({"OrganizationSummaries":organizations}));
    }
    let organization = native
        .get("OrganizationId")
        .and_then(Json::as_str)
        .ok_or("native OrganizationId absent")?;
    assert!(
        organization == "m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            || (shared && organization == "m-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
    if matches!(operation, "ListGroups" | "ListUsers" | "ListResources") {
        let items = match operation {
            "ListGroups" => "Groups",
            "ListUsers" => "Users",
            _ => "Resources",
        };
        return Ok(
            json!({items:[{"Id":"collision","Name":"collision","State":"ENABLED","Email":"collision@example.test"}]}),
        );
    }
    if matches!(
        operation,
        "ListAliases" | "ListGroupsForEntity" | "ListMailboxPermissions"
    ) {
        assert_eq!(native.get("EntityId"), Some(&json!("collision")));
        let parent = lease
            .scope()
            .parent()
            .ok_or("native child omitted exact producer parent")?;
        assert_eq!(parent.account.as_str(), "123456789012");
        assert_eq!(parent.region.as_str(), "us-east-1");
        assert_eq!(parent.id.as_str(), correlated(&[organization, "collision"]));
        let label = match parent.kind.as_str() {
            "AWS::ApiOnly::WorkMailUser" => "user",
            "AWS::ApiOnly::WorkMailGroup" => "group",
            "AWS::ApiOnly::WorkMailResource" => "resource",
            _ => return Err("unsupported native producer kind".into()),
        };
        let local = |prefix| {
            if shared {
                local_id(prefix, "shared")
            } else {
                local_id(prefix, label)
            }
        };
        let permissions = if shared && label == "user" {
            "READ"
        } else {
            "FULL_ACCESS"
        };
        let (kind, response) = match operation {
            "ListAliases" => ("WorkMailAlias", json!({"Aliases":[local("alias")]})),
            "ListGroupsForEntity" => (
                "WorkMailEntityGroup",
                json!({"Groups":[{"GroupId":local("entity-group"),"Name":"metadata"}]}),
            ),
            _ => (
                "WorkMailMailboxPermission",
                json!({"Permissions":[{"GranteeId":local("grantee"),"GranteeType":"USER","PermissionValues":[permissions]}]}),
            ),
        };
        branches.push(format!("{kind}:{label}"));
        return Ok(response);
    }
    let items = match operation {
        "ListImpersonationRoles" => "Roles",
        "ListMailDomains" => "MailDomains",
        "ListAvailabilityConfigurations" => "AvailabilityConfigurations",
        "ListMobileDeviceAccessOverrides" => "Overrides",
        "ListPersonalAccessTokens" => "PersonalAccessTokenSummaries",
        "ListAccessControlRules" | "ListMobileDeviceAccessRules" => "Rules",
        "ListGroupMembers" => "Members",
        "ListResourceDelegates" => "Delegates",
        _ => return Err(format!("unexpected collision native request {operation}").into()),
    };
    Ok(json!({items:[]}))
}
