use super::*;

struct RootCase {
    operation: &'static str,
    items: &'static str,
    identity: &'static str,
    prefix: &'static str,
    kind: &'static str,
    token: &'static str,
    row: Json,
}

fn roots() -> Vec<RootCase> {
    vec![
        RootCase {
            operation: "ListAuthorizers",
            items: "authorizers",
            identity: "authorizerArn",
            prefix: "authorizer",
            kind: "AWS::IoT::Authorizer",
            token: "nextMarker",
            row: json!({"authorizerName":"visible"}),
        },
        RootCase {
            operation: "ListCACertificates",
            items: "certificates",
            identity: "certificateArn",
            prefix: "cacert",
            kind: "AWS::IoT::CACertificate",
            token: "nextMarker",
            row: json!({"certificateId":"visible","status":"ACTIVE","creationDate":0}),
        },
        RootCase {
            operation: "ListCertificates",
            items: "certificates",
            identity: "certificateArn",
            prefix: "cert",
            kind: "AWS::IoT::Certificate",
            token: "nextMarker",
            row: json!({"certificateId":"visible","status":"ACTIVE","certificateMode":"DEFAULT","creationDate":0}),
        },
        RootCase {
            operation: "ListCertificateProviders",
            items: "certificateProviders",
            identity: "certificateProviderArn",
            prefix: "certificateprovider",
            kind: "AWS::IoT::CertificateProvider",
            token: "nextToken",
            row: json!({"certificateProviderName":"visible"}),
        },
        RootCase {
            operation: "ListCommands",
            items: "commands",
            identity: "commandArn",
            prefix: "command",
            kind: "AWS::IoT::Command",
            token: "nextToken",
            row: json!({"commandId":"visible","displayName":"command","createdAt":0,"lastUpdatedAt":0,"deprecated":false,"pendingDeletion":false}),
        },
        RootCase {
            operation: "ListFleetMetrics",
            items: "fleetMetrics",
            identity: "metricArn",
            prefix: "metric",
            kind: "AWS::IoT::FleetMetric",
            token: "nextToken",
            row: json!({"metricName":"visible"}),
        },
        RootCase {
            operation: "ListMitigationActions",
            items: "actionIdentifiers",
            identity: "actionArn",
            prefix: "mitigationaction",
            kind: "AWS::IoT::MitigationAction",
            token: "nextToken",
            row: json!({"actionName":"visible","creationDate":0}),
        },
        RootCase {
            operation: "ListJobTemplates",
            items: "jobTemplates",
            identity: "jobTemplateArn",
            prefix: "jobtemplate",
            kind: "AWS::IoT::JobTemplate",
            token: "nextToken",
            row: json!({"jobTemplateId":"visible","description":"metadata","createdAt":0}),
        },
        RootCase {
            operation: "ListPackages",
            items: "packageSummaries",
            identity: "packageName",
            prefix: "",
            kind: "AWS::IoT::SoftwarePackage",
            token: "nextToken",
            row: json!({"defaultVersionName":"v1","creationDate":0,"lastModifiedDate":0}),
        },
        RootCase {
            operation: "ListProvisioningTemplates",
            items: "templates",
            identity: "templateArn",
            prefix: "provisioningtemplate",
            kind: "AWS::IoT::ProvisioningTemplate",
            token: "nextToken",
            row: json!({"templateName":"visible","enabled":true,"creationDate":0}),
        },
        RootCase {
            operation: "ListScheduledAudits",
            items: "scheduledAudits",
            identity: "scheduledAuditArn",
            prefix: "scheduledaudit",
            kind: "AWS::IoT::ScheduledAudit",
            token: "nextToken",
            row: json!({"scheduledAuditName":"visible","frequency":"DAILY"}),
        },
        RootCase {
            operation: "ListStreams",
            items: "streams",
            identity: "streamArn",
            prefix: "stream",
            kind: "AWS::IoT::Stream",
            token: "nextToken",
            row: json!({"streamId":"visible","streamVersion":1,"description":"metadata"}),
        },
        RootCase {
            operation: "ListTopicRules",
            items: "rules",
            identity: "ruleArn",
            prefix: "rule",
            kind: "AWS::IoT::TopicRule",
            token: "nextToken",
            row: json!({"ruleName":"visible","topicPattern":"sensors/#","ruleDisabled":true,"createdAt":0}),
        },
        RootCase {
            operation: "ListOTAUpdates",
            items: "otaUpdates",
            identity: "otaUpdateArn",
            prefix: "otaupdate",
            kind: "AWS::IoT::OTAUpdate",
            token: "nextToken",
            row: json!({"otaUpdateId":"visible","creationDate":0}),
        },
        RootCase {
            operation: "ListJobs",
            items: "jobs",
            identity: "jobArn",
            prefix: "job",
            kind: "AWS::IoT::Job",
            token: "nextToken",
            row: json!({"jobId":"visible","status":"CANCELED","createdAt":0,"lastUpdatedAt":0}),
        },
    ]
}

#[tokio::test]
async fn given_empty_iot_first_pages_when_real_cursors_resume_then_every_safe_root_is_durable_and_projectable()
-> TestResult {
    let catalog = fixture::catalog()?;
    for case in roots() {
        let plan = fixture::plan(&catalog, "iot", case.operation)?;
        let mut scenario = Scenario::new(&catalog, case.operation).await?;
        scenario.seed(plan).await?;
        let first = scenario.claim(case.operation).await?;
        let (request, _) = scenario.request(plan, &first)?;
        let empty = json!({(case.items):[],(case.token):"resume2"});
        scenario
            .apply(
                &catalog,
                plan,
                first,
                &request,
                &serde_json::to_vec(&empty)?,
            )
            .await?;
        scenario = scenario.reopen().await?;
        assert!(scenario.assert_nodes(case.kind, 0).await?.is_empty());
        let second = scenario.claim(case.operation).await?;
        let (request, wire) = scenario.request(plan, &second)?;
        request_contains(
            &wire,
            if case.token == "nextMarker" {
                "marker=resume2"
            } else {
                "nextToken=resume2"
            },
        )?;
        let id = if case.identity == "packageName" {
            "visible".to_owned()
        } else {
            format!("arn:aws:iot:us-east-1:123456789012:{}/visible", case.prefix)
        };
        let mut row = case.row;
        row.as_object_mut()
            .ok_or("summary not an object")?
            .insert(case.identity.to_owned(), json!(id));
        scenario
            .apply(
                &catalog,
                plan,
                second,
                &request,
                &serde_json::to_vec(&json!({(case.items):[row]}))?,
            )
            .await?;
        let nodes = scenario.assert_nodes(case.kind, 1).await?;
        assert_eq!(nodes.first().ok_or("page-two root absent")?.2, id);
        assert_eq!(nodes.first().ok_or("page-two root absent")?.1, "us-east-1");
        if case.operation == "ListPackages" || case.operation == "ListProvisioningTemplates" {
            let (operation, items) = if case.operation == "ListPackages" {
                ("ListPackageVersions", "packageVersionSummaries")
            } else {
                ("ListProvisioningTemplateVersions", "versions")
            };
            let child = fixture::plan(&catalog, "iot", operation)?;
            let lease = scenario.claim(operation).await?;
            let (request, _) = scenario.request(child, &lease)?;
            scenario
                .apply(
                    &catalog,
                    child,
                    lease,
                    &request,
                    &serde_json::to_vec(&json!({items:[]}))?,
                )
                .await?;
        }
        assert!(
            scenario
                .store
                .claim(&scenario.sweep, 1000, 10000)
                .await?
                .is_none(),
            "unreviewed extra work remained"
        );
    }
    Ok(())
}

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
