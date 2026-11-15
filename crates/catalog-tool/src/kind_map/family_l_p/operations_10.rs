use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "pcs", operation: "ListQueues", kind: "AWS::PCS::Queue" },
    OperationMapping { service: "personalize", operation: "ListCampaigns", kind: "AWS::Personalize::Campaign" },
    OperationMapping { service: "personalize", operation: "ListDatasetGroups", kind: "AWS::Personalize::DatasetGroup" },
    OperationMapping { service: "personalize", operation: "ListDatasets", kind: "AWS::Personalize::Dataset" },
    OperationMapping { service: "personalize", operation: "ListEventTrackers", kind: "AWS::Personalize::EventTracker" },
    OperationMapping { service: "personalize", operation: "ListFilters", kind: "AWS::Personalize::Filter" },
    OperationMapping { service: "personalize", operation: "ListRecommenders", kind: "AWS::Personalize::Recommender" },
    OperationMapping { service: "personalize", operation: "ListSchemas", kind: "AWS::Personalize::Schema" },
    OperationMapping { service: "personalize", operation: "ListSolutions", kind: "AWS::Personalize::Solution" },
    OperationMapping { service: "personalize", operation: "ListMetricAttributions", kind: "AWS::Personalize::MetricAttribution" },
    OperationMapping { service: "pinpoint-email", operation: "ListConfigurationSets", kind: "AWS::PinpointEmail::ConfigurationSet" },
    OperationMapping { service: "pinpoint-email", operation: "ListDedicatedIpPools", kind: "AWS::PinpointEmail::DedicatedIpPool" },
    OperationMapping { service: "pinpoint-email", operation: "ListEmailIdentities", kind: "AWS::PinpointEmail::Identity" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribeConfigurationSets", kind: "AWS::SMSVOICE::ConfigurationSet" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribeOptOutLists", kind: "AWS::SMSVOICE::OptOutList" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribePhoneNumbers", kind: "AWS::SMSVOICE::PhoneNumber" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribePools", kind: "AWS::SMSVOICE::Pool" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribeProtectConfigurations", kind: "AWS::SMSVOICE::ProtectConfiguration" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribeRegistrations", kind: "AWS::SMSVOICE::Registration" },
    OperationMapping { service: "pinpoint-sms-voice-v2", operation: "DescribeSenderIds", kind: "AWS::SMSVOICE::SenderId" },
];
