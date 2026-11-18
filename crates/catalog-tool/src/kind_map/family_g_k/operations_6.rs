use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "iam",
        operation: "ListRoles",
        kind: "AWS::IAM::Role",
    },
    OperationMapping {
        service: "iam",
        operation: "GetRole",
        kind: "AWS::IAM::Role",
    },
    OperationMapping {
        service: "iot",
        operation: "ListSecurityProfiles",
        kind: "AWS::IoT::SecurityProfile",
    },
    OperationMapping {
        service: "iottwinmaker",
        operation: "ListEntities",
        kind: "AWS::IoTTwinMaker::Entity",
    },
    OperationMapping {
        service: "kinesis",
        operation: "ListStreamConsumers",
        kind: "AWS::Kinesis::StreamConsumer",
    },
    OperationMapping {
        service: "kinesis",
        operation: "DescribeStreamSummary",
        kind: "AWS::Kinesis::Stream",
    },
    OperationMapping {
        service: "kinesis-analytics-v2",
        operation: "ListApplications",
        kind: "AWS::KinesisAnalyticsV2::Application",
    },
    OperationMapping {
        service: "kinesis-analytics-v2",
        operation: "ListSnapshots",
        kind: "AWS::KinesisAnalyticsV2::Snapshot",
    },
    OperationMapping {
        service: "kinesis-video",
        operation: "ListStreams",
        kind: "AWS::KinesisVideo::Stream",
    },
    OperationMapping {
        service: "kinesis-video",
        operation: "ListSignalingChannels",
        kind: "AWS::KinesisVideo::SignalingChannel",
    },
];
