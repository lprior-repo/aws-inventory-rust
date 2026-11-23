use super::*;
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
