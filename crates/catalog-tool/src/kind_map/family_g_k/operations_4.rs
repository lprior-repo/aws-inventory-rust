use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "iotfleetwise", operation: "ListDecoderManifests", kind: "AWS::IoTFleetWise::DecoderManifest" },
    OperationMapping { service: "iotfleetwise", operation: "ListVehicles", kind: "AWS::IoTFleetWise::Vehicle" },
    OperationMapping { service: "iotfleetwise", operation: "ListCampaigns", kind: "AWS::IoTFleetWise::Campaign" },
    OperationMapping { service: "iotfleetwise", operation: "ListStateTemplates", kind: "AWS::IoTFleetWise::StateTemplate" },
    OperationMapping { service: "iotsecuretunneling", operation: "ListTunnels", kind: "AWS::IoTSecureTunneling::Tunnel" },
    OperationMapping { service: "iotsitewise", operation: "ListAssetModels", kind: "AWS::IoTSiteWise::AssetModel" },
    OperationMapping { service: "iotsitewise", operation: "ListAssets", kind: "AWS::IoTSiteWise::Asset" },
    OperationMapping { service: "iotsitewise", operation: "ListGateways", kind: "AWS::IoTSiteWise::Gateway" },
    OperationMapping { service: "iotsitewise", operation: "ListPortals", kind: "AWS::IoTSiteWise::Portal" },
    OperationMapping { service: "iotsitewise", operation: "ListComputationModels", kind: "AWS::IoTSiteWise::ComputationModel" },
    OperationMapping { service: "iotthingsgraph", operation: "SearchFlowTemplates", kind: "AWS::IoTThingsGraph::FlowTemplate" },
    OperationMapping { service: "iotthingsgraph", operation: "SearchSystemTemplates", kind: "AWS::ApiOnly::IoTThingsGraphSystemTemplate" },
    OperationMapping { service: "iotthingsgraph", operation: "SearchSystemInstances", kind: "AWS::ApiOnly::IoTThingsGraphSystemInstance" },
    OperationMapping { service: "iottwinmaker", operation: "ListWorkspaces", kind: "AWS::IoTTwinMaker::Workspace" },
    OperationMapping { service: "iottwinmaker", operation: "GetWorkspace", kind: "AWS::IoTTwinMaker::Workspace" },
    OperationMapping { service: "ivs", operation: "ListChannels", kind: "AWS::IVS::Channel" },
    OperationMapping { service: "ivs", operation: "ListRecordingConfigurations", kind: "AWS::IVS::RecordingConfiguration" },
    OperationMapping { service: "ivs", operation: "ListPlaybackKeyPairs", kind: "AWS::IVS::PlaybackKeyPair" },
    OperationMapping { service: "ivs", operation: "ListPlaybackRestrictionPolicies", kind: "AWS::IVS::PlaybackRestrictionPolicy" },
    OperationMapping { service: "ivs-realtime", operation: "ListStages", kind: "AWS::IVS::Stage" },
];
