use super::*;

struct RootCase {
    service: &'static str, operation: &'static str, kind: &'static str,
    items: &'static str, token: &'static str, wire_token: &'static str,
    rows: [Json; 2], ids: [String; 2],
}

fn roots() -> Vec<RootCase> {
    let managed = "arn:aws:iotmanagedintegrations:us-east-1:123456789012";
    let wireless = "arn:aws:iotwireless:us-east-1:123456789012";
    let ivs = "arn:aws:ivs:us-east-1:123456789012";
    vec![
        RootCase { service:"iot-managed-integrations", operation:"ListAccountAssociations", kind:"AWS::ApiOnly::IoTManagedIntegrationsAccountAssociation", items:"Items", token:"NextToken", wire_token:"NextToken",
            rows:[json!({"Arn":format!("{managed}:account-association/first"),"AccountAssociationId":"first","AssociationState":"ASSOCIATION_SUCCEEDED","Name":"first"}),json!({"Arn":format!("{managed}:account-association/second"),"AccountAssociationId":"second","AssociationState":"ASSOCIATION_SUCCEEDED","Name":"second"})],
            ids:[format!("{managed}:account-association/first"),format!("{managed}:account-association/second")] },
        RootCase { service:"iot-managed-integrations", operation:"ListDestinations", kind:"AWS::ApiOnly::IoTManagedIntegrationsDestination", items:"DestinationList", token:"NextToken", wire_token:"NextToken",
            rows:[json!({"Name":"first","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared","DeliveryDestinationType":"KINESIS"}),json!({"Name":"second","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared","DeliveryDestinationType":"KINESIS"})], ids:["first".into(),"second".into()] },
        RootCase { service:"iot-managed-integrations", operation:"ListEventLogConfigurations", kind:"AWS::ApiOnly::IoTManagedIntegrationsEventLogConfiguration", items:"EventLogConfigurationList", token:"NextToken", wire_token:"NextToken",
            rows:[json!({"Id":"first","EventLogLevel":"INFO"}),json!({"Id":"second","EventLogLevel":"ERROR"})], ids:["first".into(),"second".into()] },
        RootCase { service:"iot-managed-integrations", operation:"ListNotificationConfigurations", kind:"AWS::ApiOnly::IoTManagedIntegrationsNotificationConfiguration", items:"NotificationConfigurationList", token:"NextToken", wire_token:"NextToken",
            rows:[json!({"EventType":"DEVICE_COMMAND","DestinationName":"first"}),json!({"EventType":"DEVICE_STATE","DestinationName":"second"})], ids:["DEVICE_COMMAND".into(),"DEVICE_STATE".into()] },
        RootCase { service:"iot-managed-integrations", operation:"ListOtaTaskConfigurations", kind:"AWS::ApiOnly::IoTManagedIntegrationsOtaTaskConfiguration", items:"Items", token:"NextToken", wire_token:"NextToken",
            rows:[json!({"TaskConfigurationId":"first","Name":"first","CreatedAt":1700000000}),json!({"TaskConfigurationId":"second","Name":"second","CreatedAt":1700000001})], ids:["first".into(),"second".into()] },
        RootCase { service:"iot-managed-integrations", operation:"ListOtaTasks", kind:"AWS::ApiOnly::IoTManagedIntegrationsOtaTask", items:"Tasks", token:"NextToken", wire_token:"NextToken",
            rows:[json!({"TaskArn":format!("{managed}:ota-task/first"),"TaskId":"first","TaskConfigurationId":"first","Status":"IN_PROGRESS"}),json!({"TaskArn":format!("{managed}:ota-task/second"),"TaskId":"second","TaskConfigurationId":"second","Status":"COMPLETED"})], ids:[format!("{managed}:ota-task/first"),format!("{managed}:ota-task/second")] },
        RootCase { service:"iot-wireless", operation:"ListNetworkAnalyzerConfigurations", kind:"AWS::IoTWireless::NetworkAnalyzerConfiguration", items:"NetworkAnalyzerConfigurationList", token:"NextToken", wire_token:"nextToken",
            rows:[json!({"Arn":format!("{wireless}:NetworkAnalyzerConfiguration/first"),"Name":"first"}),json!({"Arn":format!("{wireless}:NetworkAnalyzerConfiguration/second"),"Name":"second"})], ids:[format!("{wireless}:NetworkAnalyzerConfiguration/first"),format!("{wireless}:NetworkAnalyzerConfiguration/second")] },
        RootCase { service:"iot-wireless", operation:"ListWirelessDeviceImportTasks", kind:"AWS::IoTWireless::WirelessDeviceImportTask", items:"WirelessDeviceImportTaskList", token:"NextToken", wire_token:"nextToken",
            rows:[json!({"Arn":format!("{wireless}:ImportTask/first"),"Id":"first","Sidewalk":{"DeviceCreationFileList":["s3://imports/first.csv"],"Role":"arn:aws:iam::123456789012:role/import","Positioning":{"DestinationName":"first"}}}),json!({"Arn":format!("{wireless}:ImportTask/second"),"Id":"second","Sidewalk":{"DeviceCreationFileList":["s3://imports/second.csv"],"Role":"arn:aws:iam::123456789012:role/import","Positioning":{"DestinationName":"second"}}})], ids:[format!("{wireless}:ImportTask/first"),format!("{wireless}:ImportTask/second")] },
        RootCase { service:"iot-wireless", operation:"ListWirelessGatewayTaskDefinitions", kind:"AWS::IoTWireless::TaskDefinition", items:"TaskDefinitions", token:"NextToken", wire_token:"nextToken",
            rows:[json!({"Arn":format!("{wireless}:WirelessGatewayTaskDefinition/first"),"Id":"first","LoRaWAN":{"CurrentVersion":{"PackageVersion":"1","Model":"GW","Station":"1"},"UpdateVersion":{"PackageVersion":"2","Model":"GW","Station":"2"}}}),json!({"Arn":format!("{wireless}:WirelessGatewayTaskDefinition/second"),"Id":"second","LoRaWAN":{"CurrentVersion":{"PackageVersion":"2"},"UpdateVersion":{"PackageVersion":"3"}}})], ids:[format!("{wireless}:WirelessGatewayTaskDefinition/first"),format!("{wireless}:WirelessGatewayTaskDefinition/second")] },
        RootCase { service:"ivs-realtime", operation:"ListPublicKeys", kind:"AWS::IVS::PublicKey", items:"publicKeys", token:"nextToken", wire_token:"nextToken",
            rows:[json!({"arn":format!("{ivs}:public-key/first"),"name":"first","tags":{"env":"test"}}),json!({"arn":format!("{ivs}:public-key/second"),"name":"second"})], ids:[format!("{ivs}:public-key/first"),format!("{ivs}:public-key/second")] },
        RootCase { service:"ivs-realtime", operation:"ListIngestConfigurations", kind:"AWS::IVS::IngestConfiguration", items:"ingestConfigurations", token:"nextToken", wire_token:"nextToken",
            rows:[json!({"arn":format!("{ivs}:ingest-configuration/first"),"name":"first","ingestProtocol":"RTMP","stageArn":"","participantId":"","state":"INACTIVE","redundantIngest":false}),json!({"arn":format!("{ivs}:ingest-configuration/second"),"name":"second","ingestProtocol":"RTMPS","stageArn":format!("{ivs}:stage/attached"),"participantId":"participant","state":"ACTIVE","redundantIngest":true})], ids:[format!("{ivs}:ingest-configuration/first"),format!("{ivs}:ingest-configuration/second")] },
        RootCase { service:"ivs-realtime", operation:"ListCompositions", kind:"AWS::ApiOnly::IVSComposition", items:"compositions", token:"nextToken", wire_token:"nextToken",
            rows:[json!({"arn":format!("{ivs}:composition/first"),"stageArn":format!("{ivs}:stage/attached"),"state":"ACTIVE","destinations":[{"id":"destination-a","state":"ACTIVE","startTime":"2026-01-01T00:00:00Z"}]}),json!({"arn":format!("{ivs}:composition/second"),"stageArn":format!("{ivs}:stage/attached"),"state":"STOPPED","destinations":[{"id":"destination-b","state":"STOPPED","endTime":"2026-01-02T00:00:00Z"}]})], ids:[format!("{ivs}:composition/first"),format!("{ivs}:composition/second")] },
    ]
}

#[tokio::test]
async fn given_owned_safe_summaries_when_pages_resume_after_empty_and_nonempty_pages_then_every_kind_is_queryable_without_details() -> TestResult {
    let catalog = catalog()?;
    for case in roots() {
        let plan = fixture::plan(&catalog, case.service, case.operation)?;
        assert_eq!(plan.kind().as_str(), case.kind);
        for empty_first in [false, true] {
            let mut scenario = Scenario::new(&catalog, "safe-integrations-roots").await?;
            scenario.seed(plan).await?;
            for ordinal in 0..2 {
                let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("root continuation absent")?;
                assert_eq!(lease.scope().operation().as_str(), case.operation);
                let (request, wire) = scenario.prepared_request(plan, &lease)?;
                assert_wire_token(&wire, if ordinal == 0 { None } else { Some("secondpage") }, case.wire_token)?;
                if wire.method() != http::Method::GET {
                    let body: Json = serde_json::from_slice(wire.body())?;
                    assert_eq!(body.get("filterByState"), None);
                    assert_eq!(body.get("filterByStageArn"), None);
                }
                let row = case.rows.get(usize::try_from(ordinal)?).ok_or("response row absent")?.clone();
                let rows = if ordinal == 0 && empty_first { Vec::new() } else { vec![row] };
                let response = page(case.items, rows, if ordinal == 0 { Some("secondpage") } else { None }, case.token);
                scenario.apply(&catalog, plan, lease, &request, response).await?;
                scenario = reopen(scenario).await?;
            }
            assert!(scenario.store.claim(&scenario.sweep, 1001, 10000).await?.is_none(), "unapproved detail was scheduled");
            let actual = saved(&scenario, case.kind).await?;
            let expected = if empty_first { vec![case.ids.get(1).ok_or("second id absent")?.clone()] } else { case.ids.to_vec() };
            assert_eq!(actual.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(), expected, "{}", case.operation);
            for (_, bytes) in actual {
                let text = String::from_utf8_lossy(&bytes);
                for forbidden in ["streamKey", "AppServerPrivateKey", "PrivateKeys", "UpdateSignature", "SigKeyCrc"] { assert!(!text.contains(forbidden)); }
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_two_named_destinations_sharing_delivery_target_when_notification_is_collected_then_graph_joins_configuration_name() -> TestResult {
    let catalog = catalog()?;
    let scenario = Scenario::new(&catalog, "notification-destination-name").await?;
    let destinations = fixture::plan(&catalog, "iot-managed-integrations", "ListDestinations")?;
    let notifications = fixture::plan(&catalog, "iot-managed-integrations", "ListNotificationConfigurations")?;
    scenario.seed(destinations).await?;
    scenario.seed(notifications).await?;
    for _ in 0..2 {
        let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("configuration lease absent")?;
        let plan = fixture::plan(&catalog, "iot-managed-integrations", lease.scope().operation().as_str())?;
        let (request, _) = scenario.prepared_request(plan, &lease)?;
        let response = if plan.operation_name() == "ListDestinations" {
            json!({"DestinationList":[{"Name":"first","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared"},{"Name":"second","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared"}]})
        } else { json!({"NotificationConfigurationList":[{"EventType":"DEVICE_STATE","DestinationName":"second"}]}) };
        scenario.apply(&catalog, plan, lease, &request, response).await?;
    }
    let destination_rows = saved(&scenario, destinations.kind().as_str()).await?;
    assert_eq!(destination_rows.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), ["first", "second"]);
    let edges = scenario.store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 1);
    let edge = edges.edges.first().ok_or("destination edge absent")?;
    assert_eq!(edge.relation.as_str(), "DELIVERS_TO");
    assert_eq!(edge.from.id.as_str(), "DEVICE_STATE");
    assert_eq!(edge.to.id.as_str(), "second");
    assert_eq!(edge.to.kind, *destinations.kind());
    Ok(())
}
