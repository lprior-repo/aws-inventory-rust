use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "vpc-lattice", operation: "ListDomainVerifications", kind: "AWS::VpcLattice::DomainVerification" },
    OperationMapping { service: "vpc-lattice", operation: "ListResourceConfigurations", kind: "AWS::VpcLattice::ResourceConfiguration" },
    OperationMapping { service: "vpc-lattice", operation: "ListResourceGateways", kind: "AWS::VpcLattice::ResourceGateway" },
    OperationMapping { service: "vpc-lattice", operation: "ListServiceNetworkResourceAssociations", kind: "AWS::VpcLattice::ServiceNetworkResourceAssociation" },
    OperationMapping { service: "vpc-lattice", operation: "ListServiceNetworks", kind: "AWS::VpcLattice::ServiceNetwork" },
    OperationMapping { service: "vpc-lattice", operation: "ListServices", kind: "AWS::VpcLattice::Service" },
    OperationMapping { service: "vpc-lattice", operation: "ListTargetGroups", kind: "AWS::VpcLattice::TargetGroup" },
    OperationMapping { service: "waf", operation: "ListByteMatchSets", kind: "AWS::WAF::ByteMatchSet" },
    OperationMapping { service: "waf", operation: "ListGeoMatchSets", kind: "AWS::ApiOnly::WAFGeoMatchSet" },
    OperationMapping { service: "waf", operation: "ListIPSets", kind: "AWS::WAF::IPSet" },
    OperationMapping { service: "waf", operation: "ListRateBasedRules", kind: "AWS::ApiOnly::WAFRateBasedRule" },
    OperationMapping { service: "waf", operation: "ListRegexMatchSets", kind: "AWS::ApiOnly::WAFRegexMatchSet" },
    OperationMapping { service: "waf", operation: "ListRegexPatternSets", kind: "AWS::ApiOnly::WAFRegexPatternSet" },
    OperationMapping { service: "waf", operation: "ListRuleGroups", kind: "AWS::ApiOnly::WAFRuleGroup" },
    OperationMapping { service: "waf", operation: "ListRules", kind: "AWS::WAF::Rule" },
    OperationMapping { service: "waf", operation: "ListSizeConstraintSets", kind: "AWS::WAF::SizeConstraintSet" },
    OperationMapping { service: "waf", operation: "ListSqlInjectionMatchSets", kind: "AWS::WAF::SqlInjectionMatchSet" },
    OperationMapping { service: "waf", operation: "ListWebACLs", kind: "AWS::WAF::WebACL" },
    OperationMapping { service: "waf", operation: "ListXssMatchSets", kind: "AWS::WAF::XssMatchSet" },
    OperationMapping { service: "waf-regional", operation: "ListByteMatchSets", kind: "AWS::WAFRegional::ByteMatchSet" },
];
