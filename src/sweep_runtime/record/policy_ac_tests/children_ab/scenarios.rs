use super::*;
fn arn(service: &str, suffix: &str) -> String {
    format!("arn:aws:{service}:us-east-1:123456789012:{suffix}")
}

fn scoped(parent: &str, child: &str) -> String {
    format!("{}:{parent}|{}:{child}", parent.len(), child.len())
}

#[tokio::test]
async fn given_two_workspaces_when_empty_first_child_pages_resume_then_both_amp_child_kinds_persist()
-> TestResult {
    let workspaces = ["a", "b"].map(|name| {
        json!({"workspaceId":name,"arn":arn("aps", &format!("workspace/{name}")),
        "status":{"statusCode":"ACTIVE"},"createdAt":0})
    });
    let mut pages = vec![
        Page::new(
            "ListWorkspaces",
            json!({}),
            json!({"workspaces":workspaces}),
            vec![arn("aps", "workspace/a"), arn("aps", "workspace/b")],
        )
        .rest("/workspaces".to_owned(), None),
    ];
    for workspace in ["a", "b"] {
        let parent = arn("aps", &format!("workspace/{workspace}"));
        for (operation, items, suffix) in [
            (
                "ListAnomalyDetectors",
                "anomalyDetectors",
                "anomalydetectors",
            ),
            (
                "ListRuleGroupsNamespaces",
                "ruleGroupsNamespaces",
                "rulegroupsnamespaces",
            ),
        ] {
            let id = arn("aps", &format!("workspace/{workspace}/{suffix}/same"));
            let mut item = json!({"arn":id,"status":{"statusCode":"ACTIVE"},"createdAt":0,"modifiedAt":0,"tags":{"owner":"inventory"}});
            let fields = item
                .as_object_mut()
                .ok_or("fixture item is not an object")?;
            if operation == "ListAnomalyDetectors" {
                fields.insert("anomalyDetectorId".to_owned(), json!("same"));
                fields.insert("alias".to_owned(), json!("same"));
            } else {
                fields.insert("name".to_owned(), json!("same"));
            }
            pages.extend(later_pages(
                operation,
                json!({"workspaceId":workspace}),
                &parent,
                items,
                "nextToken",
                item,
                id,
                &format!("/workspaces/{workspace}/{suffix}"),
                Some("nextToken"),
                &[],
            ));
        }
    }
    consume(
        "amp",
        "2020-08-01",
        &["ListWorkspaces"],
        &[
            ("ListAnomalyDetectors", "AWS::APS::AnomalyDetector"),
            ("ListRuleGroupsNamespaces", "AWS::APS::RuleGroupsNamespace"),
        ],
        &[
            "DescribeRuleGroupsNamespace",
            "CreateAnomalyDetector",
            "PutRuleGroupsNamespace",
        ],
        pages,
    )
    .await
}

fn mesh_ref(mesh: &str, field: Option<&str>, suffix: &str) -> Value {
    let mut item = json!({"meshName":mesh,"meshOwner":"123456789012","resourceOwner":"123456789012",
        "arn":arn("appmesh", suffix),"version":1,"createdAt":0,"lastUpdatedAt":0});
    if let Some(field) = field {
        if let Some(obj) = item.as_object_mut() {
            obj.insert(field.to_owned(), json!("same"));
        }
    }
    item
}

#[tokio::test]
async fn given_equal_router_and_gateway_names_in_two_meshes_when_routes_resume_then_parent_pairs_never_cross()
-> TestResult {
    let mut pages = vec![
        Page::new(
            "ListMeshes",
            json!({}),
            json!({"meshes":[mesh_ref("a", None, "mesh/a"),mesh_ref("b", None, "mesh/b")]}),
            vec![arn("appmesh", "mesh/a"), arn("appmesh", "mesh/b")],
        )
        .rest("/v20190125/meshes".to_owned(), None),
    ];
    for mesh in ["a", "b"] {
        for (
            parent_op,
            parent_items,
            field,
            segment,
            operation,
            items,
            child_field,
            child_segment,
        ) in [
            (
                "ListVirtualGateways",
                "virtualGateways",
                "virtualGatewayName",
                "virtualGateway",
                "ListGatewayRoutes",
                "gatewayRoutes",
                "gatewayRouteName",
                "gatewayRoutes",
            ),
            (
                "ListVirtualRouters",
                "virtualRouters",
                "virtualRouterName",
                "virtualRouter",
                "ListRoutes",
                "routes",
                "routeName",
                "routes",
            ),
        ] {
            let parent_suffix = format!("mesh/{mesh}/{segment}/same");
            let parent = arn("appmesh", &parent_suffix);
            pages.push(
                Page::new(
                    parent_op,
                    json!({"meshName":mesh}),
                    json!({parent_items:[mesh_ref(mesh, Some(field), &parent_suffix)]}),
                    vec![parent.clone()],
                )
                .under(&arn("appmesh", &format!("mesh/{mesh}")))
                .rest(format!("/v20190125/meshes/{mesh}/{segment}s"), None),
            );
            let id = arn("appmesh", &format!("{parent_suffix}/{child_segment}/same"));
            let mut item = mesh_ref(
                mesh,
                Some(field),
                &format!("{parent_suffix}/{child_segment}/same"),
            );
            item.as_object_mut()
                .ok_or("fixture item is not an object")?
                .insert(child_field.to_owned(), json!("same"));
            pages.extend(later_pages(
                operation,
                json!({"meshName":mesh,field:"same","meshOwner":"123456789012"}),
                &parent,
                items,
                "nextToken",
                item,
                id,
                &format!(
                    "/v20190125/meshes/{mesh}/{segment}/same/{child_segment}?meshOwner=123456789012"
                ),
                Some("nextToken"),
                &[],
            ));
        }
    }
    consume(
        "app-mesh",
        "2019-01-25",
        &["ListMeshes"],
        &[
            ("ListGatewayRoutes", "AWS::AppMesh::GatewayRoute"),
            ("ListRoutes", "AWS::AppMesh::Route"),
        ],
        &["DescribeGatewayRoute", "DescribeRoute", "CreateRoute"],
        pages,
    )
    .await
}

#[tokio::test]
async fn given_two_backup_and_restore_plans_when_equal_selection_names_resume_then_selected_parent_keys_prevent_overwrites()
-> TestResult {
    let mut pages = vec![
        Page::new("ListBackupPlans", json!({}), json!({"BackupPlansList":[
            {"BackupPlanId":"a","BackupPlanArn":arn("backup", "backup-plan:a")}, {"BackupPlanId":"b","BackupPlanArn":arn("backup", "backup-plan:b")}]}),
            vec![arn("backup", "backup-plan:a"),arn("backup", "backup-plan:b")]).rest("/backup/plans".to_owned(), None),
        Page::new("ListRestoreTestingPlans", json!({}), json!({"RestoreTestingPlans":[
            {"RestoreTestingPlanName":"a","RestoreTestingPlanArn":arn("backup", "restore-testing-plan:a"),"CreationTime":0,"ScheduleExpression":"cron(0 5 ? * * *)"},
            {"RestoreTestingPlanName":"b","RestoreTestingPlanArn":arn("backup", "restore-testing-plan:b"),"CreationTime":0,"ScheduleExpression":"cron(0 5 ? * * *)"}]}),
            vec![arn("backup", "restore-testing-plan:a"),arn("backup", "restore-testing-plan:b")]).rest("/restore-testing/plans".to_owned(), None),
    ];
    for plan in ["a", "b"] {
        pages.extend(later_pages(
            "ListBackupSelections",
            json!({"BackupPlanId":plan}),
            &arn("backup", &format!("backup-plan:{plan}")),
            "BackupSelectionsList",
            "NextToken",
            json!({"BackupPlanId":plan,"SelectionId":"same","SelectionName":"same","CreationDate":0,
                "IamRoleArn":"arn:aws:iam::123456789012:role/backup"}),
            scoped(plan, "same"),
            &format!("/backup/plans/{plan}/selections"),
            Some("nextToken"),
            &[],
        ));
        pages.extend(later_pages("ListRestoreTestingSelections", json!({"RestoreTestingPlanName":plan}), &arn("backup", &format!("restore-testing-plan:{plan}")),
            "RestoreTestingSelections", "NextToken", json!({"RestoreTestingPlanName":plan,"RestoreTestingSelectionName":"same","CreationTime":0,
                "IamRoleArn":"arn:aws:iam::123456789012:role/backup","ProtectedResourceType":"EBS","ValidationWindowHours":1}),
            scoped(plan, "same"), &format!("/restore-testing/plans/{plan}/selections"), Some("NextToken"), &[]));
    }
    consume(
        "backup",
        "2018-11-15",
        &["ListBackupPlans", "ListRestoreTestingPlans"],
        &[
            ("ListBackupSelections", "AWS::Backup::BackupSelection"),
            (
                "ListRestoreTestingSelections",
                "AWS::Backup::RestoreTestingSelection",
            ),
        ],
        &["GetBackupSelection", "StartBackupJob"],
        pages,
    )
    .await
}

#[tokio::test]
async fn given_two_agents_and_knowledge_bases_when_shared_child_ids_resume_then_aliases_and_data_sources_keep_their_owner()
-> TestResult {
    let mut pages = vec![
        Page::new("ListAgents", json!({}), json!({"agentSummaries":[
            {"agentId":"AGENT00001","agentName":"a","agentStatus":"PREPARED","updatedAt":"2026-01-01T00:00:00Z"},
            {"agentId":"AGENT00002","agentName":"b","agentStatus":"PREPARED","updatedAt":"2026-01-01T00:00:00Z"}]}), vec!["AGENT00001".to_owned(),"AGENT00002".to_owned()])
            .rest("/agents/".to_owned(), None),
        Page::new("ListKnowledgeBases", json!({}), json!({"knowledgeBaseSummaries":[
            {"knowledgeBaseId":"KB00000001","name":"a","status":"ACTIVE","updatedAt":"2026-01-01T00:00:00Z"},
            {"knowledgeBaseId":"KB00000002","name":"b","status":"ACTIVE","updatedAt":"2026-01-01T00:00:00Z"}]}), vec!["KB00000001".to_owned(),"KB00000002".to_owned()])
            .rest("/knowledgebases/".to_owned(), None),
    ];
    for agent in ["AGENT00001", "AGENT00002"] {
        pages.extend(later_pages("ListAgentAliases", json!({"agentId":agent}), agent, "agentAliasSummaries", "nextToken",
            json!({"agentAliasId":"TSTALIASID","agentAliasName":"test","agentAliasStatus":"PREPARED","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z",
                "routingConfiguration":[{"agentVersion":"DRAFT"}]}), scoped(agent, "TSTALIASID"), &format!("/agents/{agent}/agentaliases/"), None, &["agentId"]));
    }
    for knowledge_base in ["KB00000001", "KB00000002"] {
        pages.extend(later_pages("ListDataSources", json!({"knowledgeBaseId":knowledge_base}), knowledge_base, "dataSourceSummaries", "nextToken",
            json!({"knowledgeBaseId":knowledge_base,"dataSourceId":"SOURCE0001","name":"same","status":"AVAILABLE","updatedAt":"2026-01-01T00:00:00Z"}),
            scoped(knowledge_base, "SOURCE0001"), &format!("/knowledgebases/{knowledge_base}/datasources/"), None, &[]));
    }
    consume(
        "bedrock-agent",
        "2023-06-05",
        &["ListAgents", "ListKnowledgeBases"],
        &[
            ("ListAgentAliases", "AWS::Bedrock::AgentAlias"),
            ("ListDataSources", "AWS::Bedrock::DataSource"),
        ],
        &[
            "GetAgent",
            "GetDataSource",
            "GetKnowledgeBaseDocuments",
            "StartIngestionJob",
        ],
        pages,
    )
    .await
}

#[tokio::test]
async fn given_non_primary_workgroups_when_saved_queries_and_equal_statement_names_resume_then_inventory_never_fetches_sql()
-> TestResult {
    let mut pages = vec![Page::new(
        "ListWorkGroups",
        json!({}),
        json!({"WorkGroups":[{"Name":"analytics"},{"Name":"reporting"}]}),
        vec!["analytics".to_owned(), "reporting".to_owned()],
    )];
    for workgroup in ["analytics", "reporting"] {
        pages.extend(later_pages(
            "ListNamedQueries",
            json!({"WorkGroup":workgroup}),
            workgroup,
            "NamedQueryIds",
            "NextToken",
            json!("query-id"),
            scoped(workgroup, "query-id"),
            "/",
            None,
            &["WorkGroup"],
        ));
        pages.extend(later_pages(
            "ListPreparedStatements",
            json!({"WorkGroup":workgroup}),
            workgroup,
            "PreparedStatements",
            "NextToken",
            json!({"StatementName":"same","LastModifiedTime":0}),
            scoped(workgroup, "same"),
            "/",
            None,
            &["WorkGroup"],
        ));
    }
    consume(
        "athena",
        "2017-05-18",
        &["ListWorkGroups"],
        &[
            ("ListNamedQueries", "AWS::Athena::NamedQuery"),
            ("ListPreparedStatements", "AWS::Athena::PreparedStatement"),
        ],
        &[
            "GetNamedQuery",
            "GetPreparedStatement",
            "StartQueryExecution",
        ],
        pages,
    )
    .await
}

#[tokio::test]
async fn given_two_scalar_applications_when_equal_deployment_group_names_resume_then_both_qualified_groups_persist()
-> TestResult {
    let mut pages = vec![Page::new(
        "ListApplications",
        json!({}),
        json!({"applications":["A","B"]}),
        vec!["A".to_owned(), "B".to_owned()],
    )];
    for application in ["A", "B"] {
        pages.extend(later_pages(
            "ListDeploymentGroups",
            json!({"applicationName":application}),
            application,
            "deploymentGroups",
            "nextToken",
            json!("prod"),
            scoped(application, "prod"),
            "/",
            None,
            &["applicationName"],
        ));
    }
    let pages = pages
        .into_iter()
        .map(|page| page.cbor("CodeDeploy_20141006"))
        .collect();
    consume(
        "codedeploy",
        "2014-10-06",
        &["ListApplications"],
        &[("ListDeploymentGroups", "AWS::CodeDeploy::DeploymentGroup")],
        &["GetApplicationRevision", "CreateDeployment"],
        pages,
    )
    .await
}

fn escaped_label(arn: &str) -> String {
    arn.replace(':', "%3A").replace('/', "%2F")
}

#[tokio::test]
async fn given_two_bundles_when_ingestion_and_destination_pages_resume_then_request_context_keeps_each_pair_correlated()
-> TestResult {
    let mut pages = vec![
        Page::new(
            "ListAppBundles",
            json!({}),
            json!({"appBundleSummaryList":[
        {"arn":arn("appfabric", "appbundle/a")},{"arn":arn("appfabric", "appbundle/b")}]}),
            vec![
                arn("appfabric", "appbundle/a"),
                arn("appfabric", "appbundle/b"),
            ],
        )
        .rest("/appbundles".to_owned(), None),
    ];
    for name in ["a", "b"] {
        let bundle = arn("appfabric", &format!("appbundle/{name}"));
        let ingestion = arn("appfabric", &format!("appbundle/{name}/ingestion/same"));
        let destination = arn(
            "appfabric",
            &format!("appbundle/{name}/ingestion/same/ingestiondestination/same"),
        );
        pages.extend(later_pages(
            "ListIngestions",
            json!({"appBundleIdentifier":bundle}),
            &bundle,
            "ingestions",
            "nextToken",
            json!({"arn":ingestion,"app":"fixture","tenantId":"tenant","state":"enabled"}),
            ingestion.clone(),
            &format!("/appbundles/{}/ingestions", escaped_label(&bundle)),
            Some("nextToken"),
            &["appBundleIdentifier"],
        ));
        pages.extend(later_pages(
            "ListIngestionDestinations",
            json!({"appBundleIdentifier":bundle,"ingestionIdentifier":ingestion}),
            &ingestion,
            "ingestionDestinations",
            "nextToken",
            json!({"arn":destination}),
            destination.clone(),
            &format!(
                "/appbundles/{}/ingestions/{}/ingestiondestinations",
                escaped_label(&bundle),
                escaped_label(&ingestion)
            ),
            Some("nextToken"),
            &["appBundleIdentifier", "ingestionIdentifier"],
        ));
    }
    consume(
        "appfabric",
        "2023-05-19",
        &["ListAppBundles"],
        &[(
            "ListIngestionDestinations",
            "AWS::AppFabric::IngestionDestination",
        )],
        &[
            "GetIngestionDestination",
            "StartIngestion",
            "CreateAppAuthorization",
        ],
        pages,
    )
    .await
}

#[tokio::test]
async fn given_owned_domains_when_package_group_pages_resume_then_all_group_arns_persist_without_assets_or_credentials()
-> TestResult {
    let mut pages = vec![Page::new("ListDomains", json!({}), json!({"domains":[
        {"name":"a","owner":"123456789012","arn":arn("codeartifact", "domain/a"),"status":"Active","createdTime":0},
        {"name":"b","owner":"123456789012","arn":arn("codeartifact", "domain/b"),"status":"Active","createdTime":0}]}),
        vec![arn("codeartifact", "domain/a"),arn("codeartifact", "domain/b")]).rest("/v1/domains".to_owned(), None)];
    for domain in ["a", "b"] {
        let id = arn("codeartifact", &format!("package-group/{domain}/same"));
        pages.extend(later_pages("ListPackageGroups", json!({"domain":domain,"domainOwner":"123456789012"}),
            &arn("codeartifact", &format!("domain/{domain}")), "packageGroups", "nextToken", json!({"arn":id,"pattern":"/npm/*",
                "domainName":domain,"domainOwner":"123456789012","createdTime":0,"description":"metadata","contactInfo":"ops@example.test",
                "originConfiguration":{"restrictions":{"EXTERNAL_UPSTREAM":{"mode":"BLOCK","effectiveMode":"BLOCK","repositoriesCount":0}}},
                "parent":{"arn":arn("codeartifact", &format!("package-group/{domain}/root")),"pattern":"/*"}}), id.clone(),
            &format!("/v1/package-groups?domain={domain}&domain-owner=123456789012"), Some("next-token"), &[]));
    }
    consume(
        "codeartifact",
        "2018-09-22",
        &["ListDomains"],
        &[("ListPackageGroups", "AWS::CodeArtifact::PackageGroup")],
        &[
            "GetPackageVersionAsset",
            "GetPackageVersionReadme",
            "GetAuthorizationToken",
            "CreatePackageGroup",
        ],
        pages,
    )
    .await
}
