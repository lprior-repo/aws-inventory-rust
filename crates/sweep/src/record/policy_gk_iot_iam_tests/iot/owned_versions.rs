use super::*;
#[tokio::test]
async fn given_identically_named_versions_when_bound_under_two_iot_parents_then_rows_and_edges_never_collapse()
-> TestResult {
    let catalog = fixture::catalog()?;
    for (
        parent_op,
        child_op,
        parent_items,
        child_items,
        request_name,
        selected_name,
        child_kind,
        parent_kind,
    ) in [
        (
            "ListPackages",
            "ListPackageVersions",
            "packageSummaries",
            "packageVersionSummaries",
            "packageName",
            "versionName",
            "AWS::IoT::SoftwarePackageVersion",
            "AWS::IoT::SoftwarePackage",
        ),
        (
            "ListPolicies",
            "ListPolicyVersions",
            "policies",
            "policyVersions",
            "policyName",
            "versionId",
            "AWS::ApiOnly::IoTPolicyVersion",
            "AWS::IoT::Policy",
        ),
        (
            "ListProvisioningTemplates",
            "ListProvisioningTemplateVersions",
            "templates",
            "versions",
            "templateName",
            "versionId",
            "AWS::ApiOnly::IoTProvisioningTemplateVersion",
            "AWS::IoT::ProvisioningTemplate",
        ),
    ] {
        let parent = fixture::plan(&catalog, "iot", parent_op)?;
        let child = fixture::plan(&catalog, "iot", child_op)?;
        let mut scenario = Scenario::new(&catalog, child_op).await?;
        scenario.seed(parent).await?;
        let lease = scenario.claim(parent_op).await?;
        let (request, _) = scenario.request(parent, &lease)?;
        let parents = ["parent-a", "parent-b"].into_iter().map(|name| {
            if request_name == "packageName" { json!({"packageName":name}) }
            else if request_name == "templateName" { json!({"templateName":name,"templateArn":format!("arn:aws:iot:us-east-1:123456789012:provisioningtemplate/{name}")}) }
            else { json!({"policyName":name,"policyArn":format!("arn:aws:iot:us-east-1:123456789012:policy/{name}")}) }
        }).collect::<Vec<_>>();
        scenario
            .apply(
                &catalog,
                parent,
                lease,
                &request,
                &serde_json::to_vec(&json!({parent_items:parents}))?,
            )
            .await?;
        let mut bound = std::collections::BTreeSet::new();
        let paged = !child.pagination().tokens.is_empty();
        let mut resumptions = 0;
        for _ in 0..(if paged { 4 } else { 2 }) {
            let lease = scenario.claim(child_op).await?;
            let resumed = lease.token().is_some();
            let (request, wire) = scenario.request(child, &lease)?;
            let binding = child.bindings().first().ok_or("parent binding absent")?;
            let name =
                path_value(request.root(), &binding.path)?.ok_or("actual parent request absent")?;
            let ValueRef::Text(name) = name else {
                return Err("parent binding is not text".into());
            };
            bound.insert(name.to_owned());
            request_contains(&wire, name)?;
            let body = if paged && !resumed {
                json!({child_items:[],"nextToken":"resume2"})
            } else {
                if resumed {
                    resumptions += 1;
                    request_contains(&wire, "nextToken=resume2")?;
                }
                let row = if request_name == "packageName" {
                    json!({"packageName":name,"versionName":"1","status":"DRAFT"})
                } else if request_name == "templateName" {
                    json!({selected_name:1,"isDefaultVersion":true,"creationDate":0})
                } else {
                    json!({selected_name:"1","isDefaultVersion":true,"createDate":0})
                };
                json!({child_items:[row]})
            };
            scenario
                .apply(
                    &catalog,
                    child,
                    lease,
                    &request,
                    &serde_json::to_vec(&body)?,
                )
                .await?;
            scenario = scenario.reopen().await?;
        }
        assert_eq!(
            bound,
            std::collections::BTreeSet::from(["parent-a".to_owned(), "parent-b".to_owned()])
        );
        assert_eq!(resumptions, if paged { 2 } else { 0 });
        let nodes = scenario.assert_nodes(child_kind, 2).await?;
        assert_ne!(
            nodes.first().ok_or("first version absent")?.2,
            nodes.last().ok_or("second version absent")?.2
        );
        let reader = scenario.reader().await?;
        let edges: Vec<(String, String)> = sqlx::query_as("SELECT to_kind,to_id FROM edges WHERE from_kind=? AND relation='VERSION_OF' ORDER BY to_id")
            .bind(child_kind).fetch_all(&reader).await?;
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|(kind, _)| kind == parent_kind));
        assert!(
            edges
                .iter()
                .zip(["parent-a", "parent-b"])
                .all(|((_, id), name)| id.ends_with(name))
        );
        reader.close().await;
        assert!(
            scenario
                .store
                .claim(&scenario.sweep, 1000, 10000)
                .await?
                .is_none()
        );
    }
    Ok(())
}

#[test]
fn given_foreign_iot_summary_arns_when_actual_scope_is_checked_then_partition_account_and_region_fail_closed()
-> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iot", "ListCommands")?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    for arn in [
        "arn:aws:iot:us-east-1:999999999999:command/x",
        "arn:aws-cn:iot:us-east-1:123456789012:command/x",
        "arn:aws:iot:eu-west-1:123456789012:command/x",
    ] {
        let response = fixture::decode(
            plan,
            &serde_json::to_vec(&json!({"commands":[{"commandArn":arn,"commandId":"x"}]}))?,
        )?;
        let rows = item_records(&response, plan.items())?;
        assert!(
            resource_id(
                rows.first().ok_or("command row absent")?,
                plan.identity(),
                Some(IdentityContext {
                    account: &account,
                    region: &region
                })
            )
            .is_err()
        );
    }
    Ok(())
}
