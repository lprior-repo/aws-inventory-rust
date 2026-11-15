use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "managedblockchain", operation: "ListAccessors", kind: "AWS::ManagedBlockchain::Accessor" },
    OperationMapping { service: "managedblockchain", operation: "ListNetworks", kind: "AWS::ApiOnly::ManagedBlockchainNetwork" },
    OperationMapping { service: "managedblockchain", operation: "ListMembers", kind: "AWS::ManagedBlockchain::Member" },
    OperationMapping { service: "mediaconnect", operation: "ListBridges", kind: "AWS::MediaConnect::Bridge" },
    OperationMapping { service: "mediaconnect", operation: "ListFlows", kind: "AWS::MediaConnect::Flow" },
    OperationMapping { service: "mediaconnect", operation: "ListGateways", kind: "AWS::MediaConnect::Gateway" },
    OperationMapping { service: "mediaconnect", operation: "ListRouterInputs", kind: "AWS::MediaConnect::RouterInput" },
    OperationMapping { service: "mediaconnect", operation: "ListRouterNetworkInterfaces", kind: "AWS::MediaConnect::RouterNetworkInterface" },
    OperationMapping { service: "mediaconnect", operation: "ListRouterOutputs", kind: "AWS::MediaConnect::RouterOutput" },
    OperationMapping { service: "mediaconvert", operation: "ListQueues", kind: "AWS::MediaConvert::Queue" },
    OperationMapping { service: "medialive", operation: "ListChannels", kind: "AWS::MediaLive::Channel" },
    OperationMapping { service: "medialive", operation: "ListInputDevices", kind: "AWS::ApiOnly::MediaLiveInputDevice" },
    OperationMapping { service: "medialive", operation: "ListInputSecurityGroups", kind: "AWS::MediaLive::InputSecurityGroup" },
    OperationMapping { service: "medialive", operation: "ListInputs", kind: "AWS::MediaLive::Input" },
    OperationMapping { service: "medialive", operation: "ListMultiplexes", kind: "AWS::MediaLive::Multiplex" },
    OperationMapping { service: "medialive", operation: "ListNetworks", kind: "AWS::MediaLive::Network" },
    OperationMapping { service: "medialive", operation: "ListClusters", kind: "AWS::MediaLive::Cluster" },
    OperationMapping { service: "medialive", operation: "ListSdiSources", kind: "AWS::MediaLive::SdiSource" },
    OperationMapping { service: "medialive", operation: "ListSignalMaps", kind: "AWS::MediaLive::SignalMap" },
    OperationMapping { service: "medialive", operation: "ListCloudWatchAlarmTemplates", kind: "AWS::MediaLive::CloudWatchAlarmTemplate" },
];
