use super::*;
pub(super) fn cases() -> Vec<Case> {
    vec![
        Case {
            service: "notifications",
            version: "2018-05-10",
            root: "ListNotificationConfigurations",
            child: "ListEventRules",
            root_kind: "AWS::Notifications::NotificationConfiguration",
            kind: "AWS::Notifications::EventRule",
            roots: json!({"notificationConfigurations":[{"arn":"arn:aws:notifications::123456789012:configuration/111111111111111111111111111"},{"arn":"arn:aws:notifications::123456789012:configuration/222222222222222222222222222"}]}),
            parents: [
                "arn:aws:notifications::123456789012:configuration/111111111111111111111111111",
                "arn:aws:notifications::123456789012:configuration/222222222222222222222222222",
            ],
            items: "eventRules",
            identity: "arn",
            relation: "IN_CONFIGURATION",
            paginated: true,
        },
        Case {
            service: "neptune-graph",
            version: "2023-11-29",
            root: "ListGraphs",
            child: "ListPrivateGraphEndpoints",
            root_kind: "AWS::NeptuneGraph::Graph",
            kind: "AWS::NeptuneGraph::PrivateGraphEndpoint",
            roots: json!({"graphs":[{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567890","id":"g-1234567890"},{"arn":"arn:aws:neptune-graph:us-east-1:123456789012:graph/g-1234567891","id":"g-1234567891"}]}),
            parents: ["g-1234567890", "g-1234567891"],
            items: "privateGraphEndpoints",
            identity: "vpcId",
            relation: "IN_GRAPH",
            paginated: true,
        },
        Case {
            service: "lex-models-v2",
            version: "2020-08-07",
            root: "ListBots",
            child: "ListBotAliases",
            root_kind: "AWS::Lex::Bot",
            kind: "AWS::Lex::BotAlias",
            roots: json!({"botSummaries":[{"botId":"BOT0000001"},{"botId":"BOT0000002"}]}),
            parents: ["BOT0000001", "BOT0000002"],
            items: "botAliasSummaries",
            identity: "botAliasId",
            relation: "IN_BOT",
            paginated: true,
        },
        Case {
            service: "lex-models-v2",
            version: "2020-08-07",
            root: "ListBots",
            child: "ListBotVersions",
            root_kind: "AWS::Lex::Bot",
            kind: "AWS::Lex::BotVersion",
            roots: json!({"botSummaries":[{"botId":"BOT0000001"},{"botId":"BOT0000002"}]}),
            parents: ["BOT0000001", "BOT0000002"],
            items: "botVersionSummaries",
            identity: "botVersion",
            relation: "IN_BOT",
            paginated: true,
        },
        Case {
            service: "networkmonitor",
            version: "2023-08-01",
            root: "ListMonitors",
            child: "GetMonitor",
            root_kind: "AWS::NetworkMonitor::Monitor",
            kind: "AWS::NetworkMonitor::Probe",
            roots: json!({"monitors":[{"monitorArn":"arn:aws:networkmonitor:us-east-1:123456789012:monitor/one","monitorName":"one"},{"monitorArn":"arn:aws:networkmonitor:us-east-1:123456789012:monitor/two","monitorName":"two"}]}),
            parents: ["one", "two"],
            items: "probes",
            identity: "probeArn",
            relation: "IN_MONITOR",
            paginated: false,
        },
        Case {
            service: "medialive",
            version: "2017-10-14",
            root: "ListClusters",
            child: "ListChannelPlacementGroups",
            root_kind: "AWS::MediaLive::Cluster",
            kind: "AWS::MediaLive::ChannelPlacementGroup",
            roots: json!({"clusters":[{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:one","id":"one"},{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:two","id":"two"}]}),
            parents: ["one", "two"],
            items: "channelPlacementGroups",
            identity: "arn",
            relation: "IN_CLUSTER",
            paginated: true,
        },
        Case {
            service: "medialive",
            version: "2017-10-14",
            root: "ListClusters",
            child: "ListNodes",
            root_kind: "AWS::MediaLive::Cluster",
            kind: "AWS::MediaLive::Node",
            roots: json!({"clusters":[{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:one","id":"one"},{"arn":"arn:aws:medialive:us-east-1:123456789012:cluster:two","id":"two"}]}),
            parents: ["one", "two"],
            items: "nodes",
            identity: "arn",
            relation: "IN_CLUSTER",
            paginated: true,
        },
        Case {
            service: "omics",
            version: "2022-11-28",
            root: "ListAnnotationStores",
            child: "ListAnnotationStoreVersions",
            root_kind: "AWS::Omics::AnnotationStore",
            kind: "AWS::ApiOnly::OmicsAnnotationStoreVersion",
            roots: json!({"annotationStores":[{"storeArn":"arn:aws:omics:us-east-1:123456789012:annotationStore/one","name":"one","reference":{"referenceArn":"arn:aws:omics:us-east-1:123456789012:referenceStore/ref/reference/1"},"sseConfig":{}},{"storeArn":"arn:aws:omics:us-east-1:123456789012:annotationStore/two","name":"two","reference":{"referenceArn":"arn:aws:omics:us-east-1:123456789012:referenceStore/ref/reference/1"},"sseConfig":{}}]}),
            parents: ["one", "two"],
            items: "annotationStoreVersions",
            identity: "versionArn",
            relation: "VERSION_OF",
            paginated: true,
        },
        Case {
            service: "omics",
            version: "2022-11-28",
            root: "ListReferenceStores",
            child: "ListReferences",
            root_kind: "AWS::Omics::ReferenceStore",
            kind: "AWS::ApiOnly::OmicsReference",
            roots: json!({"referenceStores":[{"arn":"arn:aws:omics:us-east-1:123456789012:referenceStore/1234567890","id":"1234567890"},{"arn":"arn:aws:omics:us-east-1:123456789012:referenceStore/1234567891","id":"1234567891"}]}),
            parents: ["1234567890", "1234567891"],
            items: "references",
            identity: "arn",
            relation: "IN_REFERENCE_STORE",
            paginated: true,
        },
        Case {
            service: "omics",
            version: "2022-11-28",
            root: "ListWorkflows",
            child: "ListWorkflowVersions",
            root_kind: "AWS::Omics::Workflow",
            kind: "AWS::Omics::WorkflowVersion",
            roots: json!({"items":[{"arn":"arn:aws:omics:us-east-1:123456789012:workflow/1234567890","id":"1234567890"},{"arn":"arn:aws:omics:us-east-1:123456789012:workflow/1234567891","id":"1234567891"}]}),
            parents: ["1234567890", "1234567891"],
            items: "items",
            identity: "arn",
            relation: "VERSION_OF",
            paginated: true,
        },
        Case {
            service: "pca-connector-ad",
            version: "2018-05-10",
            root: "ListDirectoryRegistrations",
            child: "ListServicePrincipalNames",
            root_kind: "AWS::PCAConnectorAD::DirectoryRegistration",
            kind: "AWS::PCAConnectorAD::ServicePrincipalName",
            roots: json!({"DirectoryRegistrations":[{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-1111111111"},{"Arn":"arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-2222222222"}]}),
            parents: [
                "arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-1111111111",
                "arn:aws:pca-connector-ad:us-east-1:123456789012:directory-registration/d-2222222222",
            ],
            items: "ServicePrincipalNames",
            identity: "ConnectorArn",
            relation: "IN_DIRECTORY_REGISTRATION",
            paginated: true,
        },
    ]
}
pub(super) fn child_row(case: &Case, parent: &str, second: bool) -> TestResult<Json> {
    let suffix = if second { "two" } else { "one" };
    let value = match case.child {
        "ListBotAliases" => if second { "ALIAS00002" } else { "ALIAS00001" }.to_owned(),
        "ListBotVersions" => if second { "2" } else { "1" }.to_owned(),
        "ListPrivateGraphEndpoints" => {
            format!("vpc-{}", if second { "22222222" } else { "11111111" })
        }
        "ListServicePrincipalNames" => format!(
            "arn:aws:pca-connector-ad:us-east-1:123456789012:connector/{}",
            if second {
                "22222222-2222-2222-2222-222222222222"
            } else {
                "11111111-1111-1111-1111-111111111111"
            }
        ),
        "ListEventRules" => format!(
            "{parent}/rule/{}",
            if second {
                "222222222222222222222222222"
            } else {
                "111111111111111111111111111"
            }
        ),
        "GetMonitor" => {
            format!("arn:aws:networkmonitor:us-east-1:123456789012:monitor/{parent}/probe/{suffix}")
        }
        "ListChannelPlacementGroups" => format!(
            "arn:aws:medialive:us-east-1:123456789012:channelPlacementGroup:{parent}-{suffix}"
        ),
        "ListNodes" => format!("arn:aws:medialive:us-east-1:123456789012:node:{parent}-{suffix}"),
        "ListAnnotationStoreVersions" => format!(
            "arn:aws:omics:us-east-1:123456789012:annotationStore/{parent}/version/{suffix}"
        ),
        "ListReferences" => format!(
            "arn:aws:omics:us-east-1:123456789012:referenceStore/{parent}/reference/{suffix}"
        ),
        "ListWorkflowVersions" => {
            format!("arn:aws:omics:us-east-1:123456789012:workflow/{parent}/version/{suffix}")
        }
        _ => return Err("unexpected LP child fixture operation".into()),
    };
    Ok(if case.child == "ListPrivateGraphEndpoints" {
        json!({"vpcId":value,"subnetIds":["subnet-11111111"],"status":"AVAILABLE"})
    } else {
        json!({(case.identity):value})
    })
}
