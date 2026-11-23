use super::*;

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

#[tokio::test]
async fn given_identical_workmail_native_child_ids_when_one_actual_producer_retires_then_one_multiowned_child_and_the_other_exact_owner_survive()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let mut scenario = Scenario::new(&catalog, "tz-mail-shared-native-identities").await?;
    let organizations = [
        "m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "m-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ];
    for round in 0..3 {
        scenario.sweep = SweepId::new(format!("tz-mail-shared-native-identities-{round}"))?;
        scenario
            .seed(&catalog, "workmail", &["ListOrganizations"])
            .await?;
        let mut branches = Vec::new();
        while let Some(lease) = scenario.store.claim(&scenario.sweep, 1000, 10000).await? {
            let plan = catalog
                .lookup("workmail", lease.scope().operation().as_str())
                .ok_or("shared collision producer absent")?;
            let (request, wire) = scenario.request(plan, &lease)?;
            let native: Json = serde_json::from_slice(wire.body())?;
            let body = if round > 0 && plan.operation_name() == "ListUsers" {
                assert!(organizations.contains(&text(&request, "OrganizationId")?));
                json!({"Users":[]})
            } else {
                response(plan, &lease, &native, &mut branches, true)?
            };
            scenario
                .apply(&catalog, plan, lease, &request, body)
                .await?;
            scenario = scenario.reopen(&catalog).await?;
        }
        let retired = round == 2;
        let mut expected_branches = Vec::new();
        for _ in organizations {
            expected_branches.extend([
                "WorkMailAlias:group",
                "WorkMailAlias:resource",
                "WorkMailEntityGroup:group",
                "WorkMailMailboxPermission:resource",
            ]);
            if round == 0 {
                expected_branches.extend([
                    "WorkMailAlias:user",
                    "WorkMailEntityGroup:user",
                    "WorkMailMailboxPermission:user",
                ]);
            }
        }
        expected_branches.sort();
        branches.sort();
        assert_eq!(branches, expected_branches);
        scenario
            .assert_ids(
                "AWS::ApiOnly::WorkMailUser",
                if retired {
                    Vec::new()
                } else {
                    organizations
                        .iter()
                        .map(|org| correlated(&[org, "collision"]))
                        .collect()
                },
            )
            .await?;
        for (kind, parents, prefix) in [
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
            let kind = format!("AWS::ApiOnly::{kind}");
            let local = local_id(prefix, "shared");
            scenario
                .assert_ids(
                    &kind,
                    organizations
                        .iter()
                        .map(|org| correlated(&[org, "collision", &local]))
                        .collect(),
                )
                .await?;
            let mut edges = Vec::new();
            for org in organizations {
                for parent in parents {
                    if retired && *parent == "user" {
                        continue;
                    }
                    let target = match *parent {
                        "user" => "WorkMailUser",
                        "group" => "WorkMailGroup",
                        _ => "WorkMailResource",
                    };
                    edges.push((
                        correlated(&[org, "collision", &local]),
                        format!("AWS::ApiOnly::{target}"),
                        correlated(&[org, "collision"]),
                    ));
                }
            }
            scenario.assert_edges(&kind, "FOR_ENTITY", edges).await?;
        }
        assert_shared_facets(&scenario, &organizations, retired).await?;
    }
    Ok(())
}

async fn assert_shared_facets(
    scenario: &Scenario<'_, '_>,
    organizations: &[&str],
    retired: bool,
) -> TestResult {
    let reader = scenario.reader().await?;
    type Row = (String, String, String, String, String, String, Vec<u8>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT f.account,f.region,f.kind,f.id,p.parent_kind,p.parent_id,f.record FROM resource_facets f JOIN resource_parents p ON p.child_account=f.account AND p.child_region=f.region AND p.child_kind=f.kind AND p.child_id=f.id AND p.scope_key=f.source_key WHERE f.kind IN ('AWS::ApiOnly::WorkMailAlias','AWS::ApiOnly::WorkMailEntityGroup','AWS::ApiOnly::WorkMailMailboxPermission') ORDER BY f.account,f.region,f.kind,f.id,p.parent_kind,p.parent_id"
    ).fetch_all(&reader).await?;
    let mut actual = Vec::new();
    let permission_field = FIELDS
        .as_ref()
        .map_err(|error| error.clone())?
        .id("PermissionValues")
        .ok_or("pinned PermissionValues absent")?;
    for (account, region, kind, id, parent_kind, parent_id, bytes) in rows {
        if kind == "AWS::ApiOnly::WorkMailMailboxPermission" {
            let facet = Record::from_bytes(bytes)?;
            let permissions = record::path_value(facet.root(), &[permission_field])?
                .ok_or("permission facet omitted native metadata")?
                .to_value()?;
            let expected = if parent_kind == "AWS::ApiOnly::WorkMailUser" {
                "READ"
            } else {
                "FULL_ACCESS"
            };
            assert_eq!(
                permissions,
                Value::List(vec![Value::Text(expected.to_owned())])
            );
        }
        actual.push((account, region, kind, id, parent_kind, parent_id));
    }
    let mut expected = Vec::new();
    for org in organizations {
        for (kind, parents, prefix) in [
            (
                "WorkMailAlias",
                &["WorkMailUser", "WorkMailGroup", "WorkMailResource"][..],
                "alias",
            ),
            (
                "WorkMailEntityGroup",
                &["WorkMailUser", "WorkMailGroup"][..],
                "entity-group",
            ),
            (
                "WorkMailMailboxPermission",
                &["WorkMailUser", "WorkMailResource"][..],
                "grantee",
            ),
        ] {
            for parent in parents {
                if retired && *parent == "WorkMailUser" {
                    continue;
                }
                expected.push((
                    "123456789012".to_owned(),
                    "us-east-1".to_owned(),
                    format!("AWS::ApiOnly::{kind}"),
                    correlated(&[org, "collision", &local_id(prefix, "shared")]),
                    format!("AWS::ApiOnly::{parent}"),
                    correlated(&[org, "collision"]),
                ));
            }
        }
    }
    expected.sort();
    assert_eq!(actual, expected);
    if retired {
        let table: String = sqlx::query_scalar(
            "SELECT table_name FROM kinds WHERE kind='AWS::ApiOnly::WorkMailMailboxPermission'",
        )
        .fetch_one(&reader)
        .await?;
        let mut statement =
            sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT account,region,id,record FROM \"");
        statement
            .push(table.replace('"', "\"\""))
            .push("\" ORDER BY account,region,id");
        let canonical: Vec<(String, String, String, Vec<u8>)> =
            statement.build_query_as().fetch_all(&reader).await?;
        let mut keys = Vec::new();
        for (account, region, id, bytes) in canonical {
            let record = Record::from_bytes(bytes)?;
            let permissions = record::path_value(record.root(), &[permission_field])?
                .ok_or("surviving canonical row lost permissions")?
                .to_value()?;
            assert_eq!(
                permissions,
                Value::List(vec![Value::Text("FULL_ACCESS".to_owned())])
            );
            keys.push((account, region, id));
        }
        let mut expected_keys: Vec<_> = organizations
            .iter()
            .map(|org| {
                (
                    "123456789012".to_owned(),
                    "us-east-1".to_owned(),
                    correlated(&[org, "collision", &local_id("grantee", "shared")]),
                )
            })
            .collect();
        expected_keys.sort();
        assert_eq!(keys, expected_keys);
    }
    let orphaned: Vec<(String,String,String,String)> = sqlx::query_as(
        "SELECT f.account,f.region,f.kind,f.id FROM resource_facets f WHERE f.kind IN ('AWS::ApiOnly::WorkMailAlias','AWS::ApiOnly::WorkMailEntityGroup','AWS::ApiOnly::WorkMailMailboxPermission') AND NOT EXISTS (SELECT 1 FROM resource_parents p WHERE p.child_account=f.account AND p.child_region=f.region AND p.child_kind=f.kind AND p.child_id=f.id AND p.scope_key=f.source_key) ORDER BY f.account,f.region,f.kind,f.id"
    ).fetch_all(&reader).await?;
    assert!(
        orphaned.is_empty(),
        "child facets survived without their actual producer authority: {orphaned:?}"
    );
    reader.close().await;
    Ok(())
}
