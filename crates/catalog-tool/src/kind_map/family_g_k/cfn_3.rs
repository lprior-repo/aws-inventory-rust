use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::IoTFleetWise::Campaign"),
    CfnMapping::canonical("AWS::IoTFleetWise::DecoderManifest"),
    CfnMapping::canonical("AWS::IoTFleetWise::Fleet"),
    CfnMapping::canonical("AWS::IoTFleetWise::ModelManifest"),
    CfnMapping::canonical("AWS::IoTFleetWise::SignalCatalog"),
    CfnMapping::canonical("AWS::IoTFleetWise::StateTemplate"),
    CfnMapping::canonical("AWS::IoTFleetWise::Vehicle"),
    CfnMapping::canonical("AWS::IoTSecureTunneling::Tunnel"),
    CfnMapping::canonical("AWS::IoTSiteWise::Asset"),
    CfnMapping::canonical("AWS::IoTSiteWise::AssetModel"),
    CfnMapping::canonical("AWS::IoTSiteWise::ComputationModel"),
    CfnMapping::canonical("AWS::IoTSiteWise::Gateway"),
    CfnMapping::canonical("AWS::IoTSiteWise::Portal"),
    CfnMapping::canonical("AWS::IoTThingsGraph::FlowTemplate"),
    CfnMapping::canonical("AWS::IoTTwinMaker::Workspace"),
    CfnMapping::canonical("AWS::IoTWireless::Destination"),
    CfnMapping::canonical("AWS::IoTWireless::DeviceProfile"),
    CfnMapping::canonical("AWS::IoTWireless::FuotaTask"),
    CfnMapping::canonical("AWS::IoTWireless::MulticastGroup"),
    CfnMapping::canonical("AWS::IoTWireless::ServiceProfile"),
];
