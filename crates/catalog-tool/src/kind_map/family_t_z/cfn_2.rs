use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::WAF::WebACL"),
    CfnMapping::canonical("AWS::WAF::XssMatchSet"),
    CfnMapping::canonical("AWS::WAFRegional::ByteMatchSet"),
    CfnMapping::canonical("AWS::WAFRegional::GeoMatchSet"),
    CfnMapping::canonical("AWS::WAFRegional::IPSet"),
    CfnMapping::canonical("AWS::WAFRegional::RateBasedRule"),
    CfnMapping::canonical("AWS::WAFRegional::RegexPatternSet"),
    CfnMapping::canonical("AWS::WAFRegional::Rule"),
    CfnMapping::canonical("AWS::WAFRegional::SizeConstraintSet"),
    CfnMapping::canonical("AWS::WAFRegional::SqlInjectionMatchSet"),
    CfnMapping::canonical("AWS::WAFRegional::WebACL"),
    CfnMapping::canonical("AWS::WAFRegional::XssMatchSet"),
    CfnMapping::canonical("AWS::WellArchitected::Profile"),
    CfnMapping::canonical("AWS::WellArchitected::ReviewTemplate"),
    CfnMapping::canonical("AWS::WellArchitected::Workload"),
    CfnMapping::canonical("AWS::Wickr::Network"),
    CfnMapping::canonical("AWS::Wisdom::Assistant"),
    CfnMapping::canonical("AWS::Wisdom::AssistantAssociation"),
    CfnMapping::canonical("AWS::Wisdom::KnowledgeBase"),
    CfnMapping::canonical("AWS::WorkSpaces::ConnectionAlias"),
];
