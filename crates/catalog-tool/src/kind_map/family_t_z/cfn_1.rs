use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::VerifiedPermissions::Policy"),
    CfnMapping::canonical("AWS::VerifiedPermissions::PolicyStore"),
    CfnMapping::canonical("AWS::VerifiedPermissions::PolicyStoreAlias"),
    CfnMapping::canonical("AWS::VerifiedPermissions::PolicyTemplate"),
    CfnMapping::canonical("AWS::VoiceID::Domain"),
    CfnMapping::canonical("AWS::VpcLattice::DomainVerification"),
    CfnMapping::canonical("AWS::VpcLattice::Listener"),
    CfnMapping::canonical("AWS::VpcLattice::ResourceConfiguration"),
    CfnMapping::canonical("AWS::VpcLattice::ResourceGateway"),
    CfnMapping::canonical("AWS::VpcLattice::Service"),
    CfnMapping::canonical("AWS::VpcLattice::ServiceNetwork"),
    CfnMapping::canonical("AWS::VpcLattice::ServiceNetworkResourceAssociation"),
    CfnMapping::canonical("AWS::VpcLattice::ServiceNetworkServiceAssociation"),
    CfnMapping::canonical("AWS::VpcLattice::ServiceNetworkVpcAssociation"),
    CfnMapping::canonical("AWS::VpcLattice::TargetGroup"),
    CfnMapping::canonical("AWS::WAF::ByteMatchSet"),
    CfnMapping::canonical("AWS::WAF::IPSet"),
    CfnMapping::canonical("AWS::WAF::Rule"),
    CfnMapping::canonical("AWS::WAF::SizeConstraintSet"),
    CfnMapping::canonical("AWS::WAF::SqlInjectionMatchSet"),
];
