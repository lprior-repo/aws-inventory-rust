use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "waf", operation: "GetXssMatchSet", kind: "AWS::WAF::XssMatchSet" },
    OperationMapping { service: "waf-regional", operation: "GetByteMatchSet", kind: "AWS::WAFRegional::ByteMatchSet" },
    OperationMapping { service: "waf-regional", operation: "GetGeoMatchSet", kind: "AWS::WAFRegional::GeoMatchSet" },
    OperationMapping { service: "waf-regional", operation: "GetIPSet", kind: "AWS::WAFRegional::IPSet" },
    OperationMapping { service: "waf-regional", operation: "GetRateBasedRule", kind: "AWS::WAFRegional::RateBasedRule" },
    OperationMapping { service: "waf-regional", operation: "GetRegexMatchSet", kind: "AWS::ApiOnly::WAFRegionalRegexMatchSet" },
    OperationMapping { service: "waf-regional", operation: "GetRegexPatternSet", kind: "AWS::WAFRegional::RegexPatternSet" },
    OperationMapping { service: "waf-regional", operation: "GetRuleGroup", kind: "AWS::ApiOnly::WAFRegionalRuleGroup" },
    OperationMapping { service: "waf-regional", operation: "GetRule", kind: "AWS::WAFRegional::Rule" },
    OperationMapping { service: "waf-regional", operation: "GetSizeConstraintSet", kind: "AWS::WAFRegional::SizeConstraintSet" },
    OperationMapping { service: "waf-regional", operation: "GetSqlInjectionMatchSet", kind: "AWS::WAFRegional::SqlInjectionMatchSet" },
    OperationMapping { service: "waf-regional", operation: "GetWebACL", kind: "AWS::WAFRegional::WebACL" },
    OperationMapping { service: "waf-regional", operation: "GetXssMatchSet", kind: "AWS::WAFRegional::XssMatchSet" },
    OperationMapping { service: "vpc-lattice", operation: "GetDomainVerification", kind: "AWS::VpcLattice::DomainVerification" },
    OperationMapping { service: "vpc-lattice", operation: "GetResourceConfiguration", kind: "AWS::VpcLattice::ResourceConfiguration" },
    OperationMapping { service: "vpc-lattice", operation: "GetResourceGateway", kind: "AWS::VpcLattice::ResourceGateway" },
    OperationMapping { service: "vpc-lattice", operation: "GetServiceNetworkResourceAssociation", kind: "AWS::VpcLattice::ServiceNetworkResourceAssociation" },
    OperationMapping { service: "vpc-lattice", operation: "GetServiceNetwork", kind: "AWS::VpcLattice::ServiceNetwork" },
    OperationMapping { service: "vpc-lattice", operation: "GetService", kind: "AWS::VpcLattice::Service" },
    OperationMapping { service: "vpc-lattice", operation: "GetTargetGroup", kind: "AWS::VpcLattice::TargetGroup" },
];
