use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Outposts::Site"),
    CfnMapping::canonical("AWS::PCAConnectorAD::Connector"),
    CfnMapping::canonical("AWS::PCAConnectorAD::DirectoryRegistration"),
    CfnMapping::canonical("AWS::PCAConnectorSCEP::Challenge"),
    CfnMapping::canonical("AWS::PCAConnectorSCEP::Connector"),
    CfnMapping::canonical("AWS::PCS::Cluster"),
    CfnMapping::canonical("AWS::PCS::ComputeNodeGroup"),
    CfnMapping::canonical("AWS::PCS::Queue"),
    CfnMapping::canonical("AWS::PaymentCryptography::Alias"),
    CfnMapping::canonical("AWS::PaymentCryptography::Key"),
    CfnMapping::canonical("AWS::Personalize::Campaign"),
    CfnMapping::canonical("AWS::Personalize::Dataset"),
    CfnMapping::canonical("AWS::Personalize::DatasetGroup"),
    CfnMapping::canonical("AWS::Personalize::EventTracker"),
    CfnMapping::canonical("AWS::Personalize::Filter"),
    CfnMapping::canonical("AWS::Personalize::MetricAttribution"),
    CfnMapping::canonical("AWS::Personalize::Recommender"),
    CfnMapping::canonical("AWS::Personalize::Schema"),
    CfnMapping::canonical("AWS::Personalize::Solution"),
    CfnMapping::canonical("AWS::Pinpoint::App"),
];
