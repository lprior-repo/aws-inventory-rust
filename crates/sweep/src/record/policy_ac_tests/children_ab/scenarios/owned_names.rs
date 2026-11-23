use super::*;
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
        pages.extend(later_pages(
            ("ListAgentAliases", json!({"agentId":agent})),
            agent,
            ("agentAliasSummaries", "nextToken"),
            (
                json!({"agentAliasId":"TSTALIASID","agentAliasName":"test","agentAliasStatus":"PREPARED","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z",
                    "routingConfiguration":[{"agentVersion":"DRAFT"}]}),
                scoped(agent, "TSTALIASID"),
            ),
            &format!("/agents/{agent}/agentaliases/"),
            None,
            &["agentId"],
        ));
    }
    for knowledge_base in ["KB00000001", "KB00000002"] {
        pages.extend(later_pages(
            ("ListDataSources", json!({"knowledgeBaseId":knowledge_base})),
            knowledge_base,
            ("dataSourceSummaries", "nextToken"),
            (
                json!({"knowledgeBaseId":knowledge_base,"dataSourceId":"SOURCE0001","name":"same","status":"AVAILABLE","updatedAt":"2026-01-01T00:00:00Z"}),
                scoped(knowledge_base, "SOURCE0001"),
            ),
            &format!("/knowledgebases/{knowledge_base}/datasources/"),
            None,
            &[],
        ));
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
            ("ListNamedQueries", json!({"WorkGroup":workgroup})),
            workgroup,
            ("NamedQueryIds", "NextToken"),
            (json!("query-id"), scoped(workgroup, "query-id")),
            "/",
            None,
            &["WorkGroup"],
        ));
        pages.extend(later_pages(
            ("ListPreparedStatements", json!({"WorkGroup":workgroup})),
            workgroup,
            ("PreparedStatements", "NextToken"),
            (
                json!({"StatementName":"same","LastModifiedTime":0}),
                scoped(workgroup, "same"),
            ),
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
            (
                "ListDeploymentGroups",
                json!({"applicationName":application}),
            ),
            application,
            ("deploymentGroups", "nextToken"),
            (json!("prod"), scoped(application, "prod")),
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
            ("ListIngestions", json!({"appBundleIdentifier":bundle})),
            &bundle,
            ("ingestions", "nextToken"),
            (
                json!({"arn":ingestion,"app":"fixture","tenantId":"tenant","state":"enabled"}),
                ingestion.clone(),
            ),
            &format!("/appbundles/{}/ingestions", escaped_label(&bundle)),
            Some("nextToken"),
            &["appBundleIdentifier"],
        ));
        pages.extend(later_pages(
            (
                "ListIngestionDestinations",
                json!({"appBundleIdentifier":bundle,"ingestionIdentifier":ingestion}),
            ),
            &ingestion,
            ("ingestionDestinations", "nextToken"),
            (json!({"arn":destination}), destination.clone()),
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
        pages.extend(later_pages(
            ("ListPackageGroups", json!({"domain":domain,"domainOwner":"123456789012"})),
            &arn("codeartifact", &format!("domain/{domain}")),
            ("packageGroups", "nextToken"),
            (
                json!({"arn":id,"pattern":"/npm/*",
                    "domainName":domain,"domainOwner":"123456789012","createdTime":0,"description":"metadata","contactInfo":"ops@example.test",
                    "originConfiguration":{"restrictions":{"EXTERNAL_UPSTREAM":{"mode":"BLOCK","effectiveMode":"BLOCK","repositoriesCount":0}}},
                    "parent":{"arn":arn("codeartifact", &format!("package-group/{domain}/root")),"pattern":"/*"}}),
                id.clone(),
            ),
            &format!("/v1/package-groups?domain={domain}&domain-owner=123456789012"),
            Some("next-token"),
            &[],
        ));
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
