use super::*;

#[tokio::test]
async fn given_two_organizations_when_all_safe_mail_metadata_fans_out_then_native_requests_and_durable_composite_owners_remain_correlated()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let scenario = Scenario::new(&catalog, "tz-mail-completion").await?;
    scenario
        .seed(&catalog, "workmail", &["ListOrganizations"])
        .await?;
    let organizations = [
        "m-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "m-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ];
    let (scenario, seen) = scenario.drain(&catalog, "workmail", |plan, request, wire, resumed| {
        let operation = plan.operation_name();
        let native: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(wire.method(), http::Method::POST);
        assert!(wire.headers().get("x-amz-target").ok_or("WorkMail target absent")?.to_str()?.ends_with(operation));
        if operation == "ListOrganizations" {
            return Ok(json!({"OrganizationSummaries": organizations.iter().map(|id| json!({"OrganizationId":id,"Alias":"owned","State":"Active"})).collect::<Vec<_>>() }));
        }
        let organization = text(request, "OrganizationId")?;
        assert!(organizations.contains(&organization));
        assert_eq!(native.get("OrganizationId").and_then(Json::as_str), Some(organization));
        let page_number = if resumed { "2" } else { "1" };
        if !plan.pagination().tokens.is_empty() { assert_token(wire, "NextToken", resumed)?; }
        let next = !resumed && !plan.pagination().tokens.is_empty();
        let result = match operation {
            "ListGroups" | "ListUsers" | "ListResources" => {
                let (items, prefix) = match operation {
                    "ListGroups" => ("Groups", "group"), "ListUsers" => ("Users", "user"), _ => ("Resources", "resource"),
                };
                page(items, json!({"Id":format!("{prefix}-{page_number}"),"Name":"safe-metadata","Email":"metadata@example.test","State":"ENABLED"}), next, "NextToken")
            }
            "ListImpersonationRoles" => page("Roles", json!({"ImpersonationRoleId":format!("role-{page_number}"),"Name":"role","Type":"FULL_ACCESS"}), next, "NextToken"),
            "ListMailDomains" => page("MailDomains", json!({"DomainName":format!("domain-{page_number}.example.test"),"DefaultDomain":true}), next, "NextToken"),
            "ListAvailabilityConfigurations" => page("AvailabilityConfigurations", json!({"DomainName":format!("availability-{page_number}.example.test"),"ProviderType":"EWS","EwsProvider":{"EwsEndpoint":"https://calendar.example.test","EwsUsername":"service-user"}}), next, "NextToken"),
            "ListMobileDeviceAccessOverrides" => page("Overrides", json!({"UserId":"shared-user","DeviceId":format!("device-{page_number}"),"Effect":"ALLOW"}), next, "NextToken"),
            "ListPersonalAccessTokens" => page("PersonalAccessTokenSummaries", json!({"PersonalAccessTokenId":format!("token-id-{page_number}"),"Name":"summary-only","Scopes":["ews:full_access"]}), next, "NextToken"),
            "ListAccessControlRules" => page("Rules", json!({"Name":"safe-rule","Effect":"ALLOW","IpRanges":["192.0.2.0/24"]}), false, "NextToken"),
            "ListMobileDeviceAccessRules" => page("Rules", json!({"MobileDeviceAccessRuleId":"device-rule","Name":"safe-rule","Effect":"ALLOW"}), false, "NextToken"),
            "ListGroupMembers" => {
                let group = text(request, "GroupId")?;
                assert!(matches!(group, "group-1" | "group-2"));
                assert_eq!(native.get("GroupId").and_then(Json::as_str), Some(group));
                page("Members", json!({"Id":format!("member-{page_number}"),"Name":"member","Type":"USER","State":"ENABLED"}), next, "NextToken")
            }
            "ListResourceDelegates" => {
                let resource = text(request, "ResourceId")?;
                assert!(matches!(resource, "resource-1" | "resource-2"));
                assert_eq!(native.get("ResourceId").and_then(Json::as_str), Some(resource));
                page("Delegates", json!({"Id":format!("delegate-{page_number}"),"Name":"delegate","Type":"USER"}), next, "NextToken")
            }
            "ListAliases" | "ListGroupsForEntity" | "ListMailboxPermissions" => {
                let entity = text(request, "EntityId")?;
                assert_eq!(native.get("EntityId").and_then(Json::as_str), Some(entity));
                let allowed = match operation {
                    "ListAliases" => ["user-", "group-", "resource-"].as_slice(),
                    "ListGroupsForEntity" => ["user-", "group-"].as_slice(),
                    _ => ["user-", "resource-"].as_slice(),
                };
                assert!(allowed.iter().any(|prefix| entity.starts_with(prefix)), "{operation} received unsupported entity {entity}");
                match operation {
                    "ListAliases" => page("Aliases", json!(format!("alias-{page_number}@example.test")), next, "NextToken"),
                    "ListGroupsForEntity" => page("Groups", json!({"GroupId":format!("group-{page_number}"),"Name":"parent"}), next, "NextToken"),
                    _ => page("Permissions", json!({"GranteeId":format!("grantee-{page_number}"),"GranteeType":"USER","PermissionValues":["FULL_ACCESS"]}), next, "NextToken"),
                }
            }
            _ => return Err(format!("unexpected WorkMail operation {operation}").into()),
        };
        Ok(result)
    }).await?;
    assert_eq!(seen.len(), 16);
    for (kind, prefix) in [
        ("WorkMailGroup", "group"),
        ("WorkMailUser", "user"),
        ("WorkMailResource", "resource"),
        ("WorkMailImpersonationRole", "role"),
        ("WorkMailMailDomain", "domain"),
        ("WorkMailAvailabilityConfiguration", "availability"),
        ("WorkMailPersonalAccessToken", "token-id"),
    ] {
        let mut expected = Vec::new();
        for org in organizations {
            for p in ["1", "2"] {
                let local = if matches!(prefix, "domain" | "availability") {
                    format!("{prefix}-{p}.example.test")
                } else {
                    format!("{prefix}-{p}")
                };
                expected.push(correlated(&[org, &local]));
            }
        }
        scenario
            .assert_ids(&format!("AWS::ApiOnly::{kind}"), expected)
            .await?;
        let parents = organizations
            .into_iter()
            .flat_map(|org| {
                ["1", "2"].map(move |p| {
                    let local = if matches!(prefix, "domain" | "availability") {
                        format!("{prefix}-{p}.example.test")
                    } else {
                        format!("{prefix}-{p}")
                    };
                    (
                        correlated(&[org, &local]),
                        "AWS::ApiOnly::WorkMailOrganization".to_owned(),
                        org.to_owned(),
                    )
                })
            })
            .collect();
        scenario
            .assert_edges(&format!("AWS::ApiOnly::{kind}"), "IN_ORGANIZATION", parents)
            .await?;
    }
    for (kind, id) in [
        ("WorkMailAccessControlRule", "safe-rule"),
        ("WorkMailMobileDeviceAccessRule", "device-rule"),
    ] {
        let expected = organizations
            .iter()
            .map(|org| correlated(&[org, id]))
            .collect();
        scenario
            .assert_ids(&format!("AWS::ApiOnly::{kind}"), expected)
            .await?;
        scenario
            .assert_edges(
                &format!("AWS::ApiOnly::{kind}"),
                "IN_ORGANIZATION",
                organizations
                    .iter()
                    .map(|org| {
                        (
                            correlated(&[org, id]),
                            "AWS::ApiOnly::WorkMailOrganization".to_owned(),
                            (*org).to_owned(),
                        )
                    })
                    .collect(),
            )
            .await?;
    }
    let overrides = organizations
        .iter()
        .flat_map(|org| {
            ["device-1", "device-2"].map(|device| correlated(&[org, "shared-user", device]))
        })
        .collect();
    scenario
        .assert_ids(
            "AWS::ApiOnly::WorkMailMobileDeviceAccessOverride",
            overrides,
        )
        .await?;
    scenario
        .assert_edges(
            "AWS::ApiOnly::WorkMailMobileDeviceAccessOverride",
            "IN_ORGANIZATION",
            organizations
                .iter()
                .flat_map(|org| {
                    ["device-1", "device-2"].map(move |device| {
                        (
                            correlated(&[org, "shared-user", device]),
                            "AWS::ApiOnly::WorkMailOrganization".to_owned(),
                            (*org).to_owned(),
                        )
                    })
                })
                .collect(),
        )
        .await?;
    for (kind, prefixes, local_prefix) in [
        ("WorkMailAlias", &["user", "group", "resource"][..], "alias"),
        ("WorkMailEntityGroup", &["user", "group"][..], "group"),
        (
            "WorkMailMailboxPermission",
            &["user", "resource"][..],
            "grantee",
        ),
        ("WorkMailGroupMember", &["group"][..], "member"),
        ("WorkMailResourceDelegate", &["resource"][..], "delegate"),
    ] {
        let mut expected = Vec::new();
        let mut org_edges = Vec::new();
        let mut owner_edges = Vec::new();
        let mut group_edges = Vec::new();
        for org in organizations {
            for prefix in prefixes {
                for entity_page in ["1", "2"] {
                    for p in ["1", "2"] {
                        let entity = format!("{prefix}-{entity_page}");
                        let local = if local_prefix == "alias" {
                            format!("alias-{p}@example.test")
                        } else {
                            format!("{local_prefix}-{p}")
                        };
                        expected.push(correlated(&[org, &entity, &local]));
                        let from = correlated(&[org, &entity, &local]);
                        org_edges.push((
                            from.clone(),
                            "AWS::ApiOnly::WorkMailOrganization".to_owned(),
                            org.to_owned(),
                        ));
                        let target_kind = match *prefix {
                            "user" => "WorkMailUser",
                            "group" => "WorkMailGroup",
                            _ => "WorkMailResource",
                        };
                        owner_edges.push((
                            from.clone(),
                            format!("AWS::ApiOnly::{target_kind}"),
                            correlated(&[org, &entity]),
                        ));
                        if kind == "WorkMailEntityGroup" {
                            group_edges.push((
                                from,
                                "AWS::ApiOnly::WorkMailGroup".to_owned(),
                                correlated(&[org, &local]),
                            ));
                        }
                    }
                }
            }
        }
        scenario
            .assert_ids(&format!("AWS::ApiOnly::{kind}"), expected)
            .await?;
        let source_kind = format!("AWS::ApiOnly::{kind}");
        scenario
            .assert_edges(&source_kind, "IN_ORGANIZATION", org_edges)
            .await?;
        let relation = match kind {
            "WorkMailGroupMember" => "IN_GROUP",
            "WorkMailResourceDelegate" => "FOR_RESOURCE",
            _ => "FOR_ENTITY",
        };
        scenario
            .assert_edges(&source_kind, relation, owner_edges)
            .await?;
        if kind == "WorkMailEntityGroup" {
            scenario
                .assert_edges(&source_kind, "IN_GROUP", group_edges)
                .await?;
        }
    }
    Ok(())
}
