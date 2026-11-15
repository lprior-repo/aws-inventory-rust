use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::MediaLive::Input"),
    CfnMapping::canonical("AWS::MediaLive::InputSecurityGroup"),
    CfnMapping::canonical("AWS::MediaLive::Multiplex"),
    CfnMapping::canonical("AWS::MediaLive::Network"),
    CfnMapping::canonical("AWS::MediaLive::SdiSource"),
    CfnMapping::canonical("AWS::MediaLive::SignalMap"),
    CfnMapping::canonical("AWS::MediaPackage::OriginEndpoint"),
    CfnMapping::canonical("AWS::MediaPackage::PackagingConfiguration"),
    CfnMapping::canonical("AWS::MediaPackage::PackagingGroup"),
    CfnMapping::canonical("AWS::MediaPackageV2::Channel"),
    CfnMapping::canonical("AWS::MediaPackageV2::ChannelGroup"),
    CfnMapping::canonical("AWS::MediaPackageV2::OriginEndpoint"),
    CfnMapping::canonical("AWS::MediaStore::Container"),
    CfnMapping::canonical("AWS::MediaTailor::Channel"),
    CfnMapping::canonical("AWS::MediaTailor::SourceLocation"),
    CfnMapping::canonical("AWS::MemoryDB::ACL"),
    CfnMapping::canonical("AWS::MemoryDB::Cluster"),
    CfnMapping::canonical("AWS::MemoryDB::MultiRegionCluster"),
    CfnMapping::canonical("AWS::MemoryDB::ParameterGroup"),
    CfnMapping::canonical("AWS::MemoryDB::Snapshot"),
];
