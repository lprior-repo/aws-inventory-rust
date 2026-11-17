use super::*;

struct Case {
    service: &'static str, version: &'static str, root: &'static str, child: &'static str,
    root_kind: &'static str, kind: &'static str, roots: Json, parents: [&'static str; 2],
    items: &'static str, identity: &'static str, relation: &'static str, paginated: bool,
}

fn cases() -> Vec<Case> {
    vec![
        Case { service:"notifications", version:"2018-05-10", root:"ListNotificationConfigurations", child:"ListEventRules", root_kind:"AWS::Notifications::NotificationConfiguration", kind:"AWS::Notifications::EventRule", roots:json!({"notificationConfigurations":[{"arn":"arn:aws:notifications::123456789012:configuration/111111111111111111111111111"},{"arn":"arn:aws:notifications::123456789012:configuration/222222222222222222222222222"}]}), parents:["arn:aws:notifications::123456789012:configuration/111111111111111111111111111","arn:aws:notifications::123456789012:configuration/222222222222222222222222222"], items:"eventRules", identity:"arn", relation:"IN_CONFIGURATION", paginated:true },
        Case { service:"neptune-graph", version:"2023-11-29", root:"ListGraphs", child:"ListPrivateGraphEndpoints", root_kind:"AWS::NeptuneGraph::Graph", kind:"AWS::NeptuneGraph::PrivateGraphEndpoint", roots:json!({"graphs":[{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567890","id":"g-1234567890"},{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567891","id":"g-1234567891"}]}), parents:["g-1234567890","g-1234567891"], items:"privateGraphEndpoints", identity:"vpcId", relation:"IN_GRAPH", paginated:true },
        Case { service:"lex-models-v2", version:"2020-08-07", root:"ListBots", child:"ListBotAliases", root_kind:"AWS::Lex::Bot", kind:"AWS::Lex::BotAlias", roots:json!({"botSummaries":[{"botId":"BOT0000001"},{"botId":"BOT0000002"}]}), parents:["BOT0000001","BOT0000002"], items:"botAliasSummaries", identity:"botAliasId", relation:"IN_BOT", paginated:true },
        Case { service:"lex-models-v2", version:"2020-08-07", root:"ListBots", child:"ListBotVersions", root_kind:"AWS::Lex::Bot", kind:"AWS::Lex::BotVersion", roots:json!({"botSummaries":[{"botId":"BOT0000001"},{"botId":"BOT0000002"}]}), parents:["BOT0000001","BOT0000002"], items:"botVersionSummaries", identity:"botVersion", relation:"IN_BOT", paginated:true },
        Case { service:"networkmonitor", version:"2023-08-01", root:"ListMonitors", child:"GetMonitor", root_kind:"AWS::NetworkMonitor::Monitor", kind:"AWS::NetworkMonitor::Probe", roots:json!({"monitors":[{"monitorArn":"arn:aws:networkmonitor:us-east-1:123456789012:monitor/one","monitorName":"one"},{"monitorArn":"arn:aws:networkmonitor:us-east-1:123456789012:monitor/two","monitorName":"two"}]}), parents:["one","two"], items:"probes", identity:"probeArn", relation:"IN_MONITOR", paginated:false },
        Case { service:"medialive", version:"2017-10-14", root:"ListClusters", child:"ListChannelPlacementGroups", root_kind:"AWS::MediaLive::Cluster", kind:"AWS::MediaLive::ChannelPlacementGroup", roots:json!({"clusters":[{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:one","id":"one"},{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:two","id":"two"}]}), parents:["one","two"], items:"channelPlacementGroups", identity:"arn", relation:"IN_CLUSTER", paginated:true },
        Case { service:"medialive", version:"2017-10-14", root:"ListClusters", child:"ListNodes", root_kind:"AWS::MediaLive::Cluster", kind:"AWS::MediaLive::Node", roots:json!({"clusters":[{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:one","id":"one"},{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:two","id":"two"}]}), parents:["one","two"], items:"nodes", identity:"arn", relation:"IN_CLUSTER", paginated:true },
        Case { service:"omics", version:"2022-11-28", root:"ListAnnotationStores", child:"ListAnnotationStoreVersions", root_kind:"AWS::Omics::AnnotationStore", kind:"AWS::ApiOnly::OmicsAnnotationStoreVersion", roots:json!({"annotationStores":[{"storeArn":"arn:aws:omics:us-east-1:123456789012:annotationStore/one","name":"one","reference":{"referenceArn":"arn:aws:omics:us-east-1:123456789012:referenceStore/ref/reference/1"},"sseConfig":{}},{"storeArn":"arn:aws:omics:us-east-1:123456789012:annotationStore/two","name":"two","reference":{"referenceArn":"arn:aws:omics:us-east-1:123456789012:referenceStore/ref/reference/1"},"sseConfig":{}}]}), parents:["one","two"], items:"annotationStoreVersions", identity:"versionArn", relation:"VERSION_OF", paginated:true },
        Case { service:"omics", version:"2022-11-28", root:"ListReferenceStores", child:"ListReferences", root_kind:"AWS::Omics::ReferenceStore", kind:"AWS::ApiOnly::OmicsReference", roots:json!({"referenceStores":[{"arn":"arn:aws:omics:us-east-1:123456789012:referenceStore/1234567890","id":"1234567890"},{"arn":"arn:aws:omics:us-east-1:123456789012:referenceStore/1234567891","id":"1234567891"}]}), parents:["1234567890","1234567891"], items:"references", identity:"arn", relation:"IN_REFERENCE_STORE", paginated:true },
        Case { service:"omics", version:"2022-11-28", root:"ListWorkflows", child:"ListWorkflowVersions", root_kind:"AWS::Omics::Workflow", kind:"AWS::Omics::WorkflowVersion", roots:json!({"items":[{"arn":"arn:aws:omics:us-east-1:123456789012:workflow/1234567890","id":"1234567890"},{"arn":"arn:aws:omics:us-east-1:123456789012:workflow/1234567891","id":"1234567891"}]}), parents:["1234567890","1234567891"], items:"items", identity:"arn", relation:"VERSION_OF", paginated:true },
        Case { service:"pca-connector-ad", version:"2018-05-10", root:"ListDirectoryRegistrations", child:"ListServicePrincipalNames", root_kind:"AWS::PCAConnectorAD::DirectoryRegistration", kind:"AWS::PCAConnectorAD::ServicePrincipalName", roots:json!({"DirectoryRegistrations":[{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-1111111111"},{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-2222222222"}]}), parents:["arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-1111111111","arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-2222222222"], items:"ServicePrincipalNames", identity:"ConnectorArn", relation:"IN_DIRECTORY_REGISTRATION", paginated:true },
    ]
}

#[tokio::test]
async fn given_two_owned_parents_when_native_child_pages_resume_then_every_child_and_exact_parent_edge_persists() -> TestResult {
    for case in cases() {
        let bytes = compiled(case.service, case.version, &[case.root, case.child], case.root_kind)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let scenario = Scenario::new(&catalog, case.child).await?;
        scenario.seed(plan(&catalog, case.service, case.root)?).await?;
        let root_lease = scenario.claim().await?;
        assert_eq!(root_lease.scope().operation().as_str(), case.root);
        let root = plan(&catalog, case.service, case.root)?;
        let (request, wire) = scenario.request(root, &root_lease)?;
        if case.child == "ListWorkflowVersions" { assert!(native_input(&wire)?.contains("type=PRIVATE")); }
        scenario.apply(&catalog, root, root_lease, &request, case.roots.clone()).await?;
        let child = plan(&catalog, case.service, case.child)?;
        let mut expected = BTreeSet::new();
        let mut expected_edges = BTreeSet::new();
        let mut seen = BTreeSet::new();
        for _ in 0..if case.paginated { 4 } else { 2 } {
            let lease = scenario.claim().await?;
            assert_eq!(lease.scope().operation().as_str(), case.child);
            let second = lease.token().is_some();
            let owner = lease.scope().parent().ok_or("child lost discovered owner")?.id.as_str().to_owned();
            let (request, wire) = scenario.request(child, &lease)?;
            let parent = bound(child, &request)?;
            assert!(case.parents.contains(&parent.as_str()));
            assert_parent(&wire, &parent)?;
            assert_cursor(&wire, second)?;
            assert!(seen.insert((parent.clone(), second)));
            let rows = if case.paginated { vec![child_row(&case, &parent, second)?] } else {
                vec![child_row(&case, &parent, false)?, child_row(&case, &parent, true)?]
            };
            for (row, ordinal) in rows.iter().zip(if case.paginated { vec![second] } else { vec![false, true] }) {
                let value = row.get(case.identity).and_then(Json::as_str).ok_or("fixture native child identity absent")?;
                let id = if matches!(case.child, "ListBotAliases" | "ListBotVersions" | "ListPrivateGraphEndpoints" | "ListServicePrincipalNames") {
                    format!("{}:{}|{}:{}", parent.len(), parent, value.len(), value)
                } else { value.to_owned() };
                assert!(expected.insert(id.clone()), "different pages must retain distinct identities: {ordinal}");
                expected_edges.insert((id, owner.clone(), case.relation.to_owned()));
            }
            let body = json!({(case.items):rows});
            let token = if case.service == "pca-connector-ad" { "NextToken" } else { "nextToken" };
            let body = if case.paginated { page(body, token, second)? } else { body };
            scenario.apply(&catalog, child, lease, &request, body).await?;
        }
        assert_eq!(seen.len(), if case.paginated { 4 } else { 2 }, "{}", case.child);
        assert_eq!(scenario.stored(case.kind).await?, (expected, expected_edges), "{}", case.child);
    }
    Ok(())
}

fn child_row(case: &Case, parent: &str, second: bool) -> TestResult<Json> {
    let suffix = if second { "two" } else { "one" };
    let value = match case.child {
        "ListBotAliases" => if second { "ALIAS00002" } else { "ALIAS00001" }.to_owned(),
        "ListBotVersions" => if second { "2" } else { "1" }.to_owned(),
        "ListPrivateGraphEndpoints" => format!("vpc-{}", if second { "22222222" } else { "11111111" }),
        "ListServicePrincipalNames" => format!("arn:aws:pca-connector-ad:us-east-1:123456789012:connector/{}", if second { "22222222-2222-2222-2222-222222222222" } else { "11111111-1111-1111-1111-111111111111" }),
        "ListEventRules" => format!("{parent}/rule/{}", if second { "222222222222222222222222222" } else { "111111111111111111111111111" }),
        "GetMonitor" => format!("arn:aws:networkmonitor:us-east-1:123456789012:monitor/{parent}/probe/{suffix}"),
        "ListChannelPlacementGroups" => format!("arn:aws:medialive:us-east-1:123456789012:channelPlacementGroup:{parent}-{suffix}"),
        "ListNodes" => format!("arn:aws:medialive:us-east-1:123456789012:node:{parent}-{suffix}"),
        "ListAnnotationStoreVersions" => format!("arn:aws:omics:us-east-1:123456789012:annotationStore/{parent}/version/{suffix}"),
        "ListReferences" => format!("arn:aws:omics:us-east-1:123456789012:referenceStore/{parent}/reference/{suffix}"),
        "ListWorkflowVersions" => format!("arn:aws:omics:us-east-1:123456789012:workflow/{parent}/version/{suffix}"),
        _ => return Err("unexpected LP child fixture operation".into()),
    };
    // No selected parent identifiers: reviewed request-origin joins must supply them.
    Ok(if case.child == "ListPrivateGraphEndpoints" {
        json!({"vpcId":value,"subnetIds":["subnet-11111111"],"status":"AVAILABLE"})
    } else { json!({(case.identity):value}) })
}

#[tokio::test]
async fn given_pca_templates_with_shared_group_sid_when_nested_acl_pages_resume_then_template_scoped_entries_do_not_collapse() -> TestResult {
    let bytes = compiled("pca-connector-ad", "2018-05-10", &["ListConnectors", "ListTemplates", "ListTemplateGroupAccessControlEntries"], "AWS::PCAConnectorAD::Connector")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "pca-template-acl").await?;
    scenario.seed(plan(&catalog, "pca-connector-ad", "ListConnectors")?).await?;
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
            "ListConnectors" => json!({"Connectors":[{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:connector/11111111-1111-1111-1111-111111111111"}]}),
            "ListTemplates" => {
                let parent = bound(plan, &request)?;
                assert_parent(&wire, &parent)?;
                page(json!({"Templates":[{"Arn":format!("{parent}/template/{}", if second { "22222222-2222-2222-2222-222222222222" } else { "11111111-1111-1111-1111-111111111111" })}]}), "NextToken", second)?
            }
            "ListTemplateGroupAccessControlEntries" => {
                let parent = bound(plan, &request)?;
                assert_parent(&wire, &parent)?;
                let sid = if second { "S-1-5-21-2" } else { "S-1-5-21-1" };
                let id = format!("{}:{}|{}:{}", parent.len(), parent, sid.len(), sid);
                expected.insert(id.clone());
                edges.insert((id, parent.clone(), "IN_TEMPLATE".to_owned()));
                page(json!({"AccessControlEntries":[{"GroupSecurityIdentifier":sid}]}), "NextToken", second)?
            }
            _ => return Err("unexpected content or write operation scheduled".into()),
        };
        scenario.apply(&catalog, plan, lease, &request, body).await?;
    }
    assert_eq!(expected.len(), 4);
    assert_eq!(scenario.stored("AWS::PCAConnectorAD::TemplateGroupAccessControlEntry").await?, (expected, edges));
    Ok(())
}
