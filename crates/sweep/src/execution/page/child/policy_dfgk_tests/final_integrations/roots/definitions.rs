use super::*;
pub(super) fn roots() -> Vec<RootCase> {
    let managed = "arn:aws:iotmanagedintegrations:us-east-1:123456789012";
    let wireless = "arn:aws:iotwireless:us-east-1:123456789012";
    let ivs = "arn:aws:ivs:us-east-1:123456789012";
    vec![
        RootCase {
            service: "iot-managed-integrations",
            operation: "ListAccountAssociations",
            kind: "AWS::ApiOnly::IoTManagedIntegrationsAccountAssociation",
            items: "Items",
            token: "NextToken",
            wire_token: "NextToken",
            rows: [
                json!({"Arn":format!("{managed}:account-association/first"),"AccountAssociationId":"first","AssociationState":"ASSOCIATION_SUCCEEDED","Name":"first"}),
                json!({"Arn":format!("{managed}:account-association/second"),"AccountAssociationId":"second","AssociationState":"ASSOCIATION_SUCCEEDED","Name":"second"}),
            ],
            ids: [
                format!("{managed}:account-association/first"),
                format!("{managed}:account-association/second"),
            ],
        },
        RootCase {
            service: "iot-managed-integrations",
            operation: "ListDestinations",
            kind: "AWS::ApiOnly::IoTManagedIntegrationsDestination",
            items: "DestinationList",
            token: "NextToken",
            wire_token: "NextToken",
            rows: [
                json!({"Name":"first","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared","DeliveryDestinationType":"KINESIS"}),
                json!({"Name":"second","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared","DeliveryDestinationType":"KINESIS"}),
            ],
            ids: ["first".into(), "second".into()],
        },
        RootCase {
            service: "iot-managed-integrations",
            operation: "ListEventLogConfigurations",
            kind: "AWS::ApiOnly::IoTManagedIntegrationsEventLogConfiguration",
            items: "EventLogConfigurationList",
            token: "NextToken",
            wire_token: "NextToken",
            rows: [
                json!({"Id":"first","EventLogLevel":"INFO"}),
                json!({"Id":"second","EventLogLevel":"ERROR"}),
            ],
            ids: ["first".into(), "second".into()],
        },
        RootCase {
            service: "iot-managed-integrations",
            operation: "ListNotificationConfigurations",
            kind: "AWS::ApiOnly::IoTManagedIntegrationsNotificationConfiguration",
            items: "NotificationConfigurationList",
            token: "NextToken",
            wire_token: "NextToken",
            rows: [
                json!({"EventType":"DEVICE_COMMAND","DestinationName":"first"}),
                json!({"EventType":"DEVICE_STATE","DestinationName":"second"}),
            ],
            ids: ["DEVICE_COMMAND".into(), "DEVICE_STATE".into()],
        },
        RootCase {
            service: "iot-managed-integrations",
            operation: "ListOtaTaskConfigurations",
            kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTaskConfiguration",
            items: "Items",
            token: "NextToken",
            wire_token: "NextToken",
            rows: [
                json!({"TaskConfigurationId":"first","Name":"first","CreatedAt":1700000000}),
                json!({"TaskConfigurationId":"second","Name":"second","CreatedAt":1700000001}),
            ],
            ids: ["first".into(), "second".into()],
        },
        RootCase {
            service: "iot-managed-integrations",
            operation: "ListOtaTasks",
            kind: "AWS::ApiOnly::IoTManagedIntegrationsOtaTask",
            items: "Tasks",
            token: "NextToken",
            wire_token: "NextToken",
            rows: [
                json!({"TaskArn":format!("{managed}:ota-task/first"),"TaskId":"first","TaskConfigurationId":"first","Status":"IN_PROGRESS"}),
                json!({"TaskArn":format!("{managed}:ota-task/second"),"TaskId":"second","TaskConfigurationId":"second","Status":"COMPLETED"}),
            ],
            ids: [
                format!("{managed}:ota-task/first"),
                format!("{managed}:ota-task/second"),
            ],
        },
        RootCase {
            service: "iot-wireless",
            operation: "ListNetworkAnalyzerConfigurations",
            kind: "AWS::IoTWireless::NetworkAnalyzerConfiguration",
            items: "NetworkAnalyzerConfigurationList",
            token: "NextToken",
            wire_token: "nextToken",
            rows: [
                json!({"Arn":format!("{wireless}:NetworkAnalyzerConfiguration/first"),"Name":"first"}),
                json!({"Arn":format!("{wireless}:NetworkAnalyzerConfiguration/second"),"Name":"second"}),
            ],
            ids: [
                format!("{wireless}:NetworkAnalyzerConfiguration/first"),
                format!("{wireless}:NetworkAnalyzerConfiguration/second"),
            ],
        },
        RootCase {
            service: "iot-wireless",
            operation: "ListWirelessDeviceImportTasks",
            kind: "AWS::IoTWireless::WirelessDeviceImportTask",
            items: "WirelessDeviceImportTaskList",
            token: "NextToken",
            wire_token: "nextToken",
            rows: [
                json!({"Arn":format!("{wireless}:ImportTask/first"),"Id":"first","Sidewalk":{"DeviceCreationFileList":["s3://imports/first.csv"],"Role":"arn:aws:iam::123456789012:role/import","Positioning":{"DestinationName":"first"}}}),
                json!({"Arn":format!("{wireless}:ImportTask/second"),"Id":"second","Sidewalk":{"DeviceCreationFileList":["s3://imports/second.csv"],"Role":"arn:aws:iam::123456789012:role/import","Positioning":{"DestinationName":"second"}}}),
            ],
            ids: [
                format!("{wireless}:ImportTask/first"),
                format!("{wireless}:ImportTask/second"),
            ],
        },
        RootCase {
            service: "iot-wireless",
            operation: "ListWirelessGatewayTaskDefinitions",
            kind: "AWS::IoTWireless::TaskDefinition",
            items: "TaskDefinitions",
            token: "NextToken",
            wire_token: "nextToken",
            rows: [
                json!({"Arn":format!("{wireless}:WirelessGatewayTaskDefinition/first"),"Id":"first","LoRaWAN":{"CurrentVersion":{"PackageVersion":"1","Model":"GW","Station":"1"},"UpdateVersion":{"PackageVersion":"2","Model":"GW","Station":"2"}}}),
                json!({"Arn":format!("{wireless}:WirelessGatewayTaskDefinition/second"),"Id":"second","LoRaWAN":{"CurrentVersion":{"PackageVersion":"2"},"UpdateVersion":{"PackageVersion":"3"}}}),
            ],
            ids: [
                format!("{wireless}:WirelessGatewayTaskDefinition/first"),
                format!("{wireless}:WirelessGatewayTaskDefinition/second"),
            ],
        },
        RootCase {
            service: "ivs-realtime",
            operation: "ListPublicKeys",
            kind: "AWS::IVS::PublicKey",
            items: "publicKeys",
            token: "nextToken",
            wire_token: "nextToken",
            rows: [
                json!({"arn":format!("{ivs}:public-key/first"),"name":"first","tags":{"env":"test"}}),
                json!({"arn":format!("{ivs}:public-key/second"),"name":"second"}),
            ],
            ids: [
                format!("{ivs}:public-key/first"),
                format!("{ivs}:public-key/second"),
            ],
        },
        RootCase {
            service: "ivs-realtime",
            operation: "ListIngestConfigurations",
            kind: "AWS::IVS::IngestConfiguration",
            items: "ingestConfigurations",
            token: "nextToken",
            wire_token: "nextToken",
            rows: [
                json!({"arn":format!("{ivs}:ingest-configuration/first"),"name":"first","ingestProtocol":"RTMP","stageArn":"","participantId":"","state":"INACTIVE","redundantIngest":false}),
                json!({"arn":format!("{ivs}:ingest-configuration/second"),"name":"second","ingestProtocol":"RTMPS","stageArn":format!("{ivs}:stage/attached"),"participantId":"participant","state":"ACTIVE","redundantIngest":true}),
            ],
            ids: [
                format!("{ivs}:ingest-configuration/first"),
                format!("{ivs}:ingest-configuration/second"),
            ],
        },
        RootCase {
            service: "ivs-realtime",
            operation: "ListCompositions",
            kind: "AWS::ApiOnly::IVSComposition",
            items: "compositions",
            token: "nextToken",
            wire_token: "nextToken",
            rows: [
                json!({"arn":format!("{ivs}:composition/first"),"stageArn":format!("{ivs}:stage/attached"),"state":"ACTIVE","destinations":[{"id":"destination-a","state":"ACTIVE","startTime":"2026-01-01T00:00:00Z"}]}),
                json!({"arn":format!("{ivs}:composition/second"),"stageArn":format!("{ivs}:stage/attached"),"state":"STOPPED","destinations":[{"id":"destination-b","state":"STOPPED","endTime":"2026-01-02T00:00:00Z"}]}),
            ],
            ids: [
                format!("{ivs}:composition/first"),
                format!("{ivs}:composition/second"),
            ],
        },
    ]
}
