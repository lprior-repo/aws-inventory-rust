use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "waf-regional", operation: "ListGeoMatchSets", kind: "AWS::WAFRegional::GeoMatchSet" },
    OperationMapping { service: "waf-regional", operation: "ListIPSets", kind: "AWS::WAFRegional::IPSet" },
    OperationMapping { service: "waf-regional", operation: "ListRateBasedRules", kind: "AWS::WAFRegional::RateBasedRule" },
    OperationMapping { service: "waf-regional", operation: "ListRegexMatchSets", kind: "AWS::ApiOnly::WAFRegionalRegexMatchSet" },
    OperationMapping { service: "waf-regional", operation: "ListRegexPatternSets", kind: "AWS::WAFRegional::RegexPatternSet" },
    OperationMapping { service: "waf-regional", operation: "ListRuleGroups", kind: "AWS::ApiOnly::WAFRegionalRuleGroup" },
    OperationMapping { service: "waf-regional", operation: "ListRules", kind: "AWS::WAFRegional::Rule" },
    OperationMapping { service: "waf-regional", operation: "ListSizeConstraintSets", kind: "AWS::WAFRegional::SizeConstraintSet" },
    OperationMapping { service: "waf-regional", operation: "ListSqlInjectionMatchSets", kind: "AWS::WAFRegional::SqlInjectionMatchSet" },
    OperationMapping { service: "waf-regional", operation: "ListWebACLs", kind: "AWS::WAFRegional::WebACL" },
    OperationMapping { service: "waf-regional", operation: "ListXssMatchSets", kind: "AWS::WAFRegional::XssMatchSet" },
    OperationMapping { service: "wellarchitected", operation: "ListProfiles", kind: "AWS::WellArchitected::Profile" },
    OperationMapping { service: "wellarchitected", operation: "ListReviewTemplates", kind: "AWS::WellArchitected::ReviewTemplate" },
    OperationMapping { service: "wellarchitected", operation: "ListWorkloads", kind: "AWS::WellArchitected::Workload" },
    OperationMapping { service: "wickr", operation: "ListNetworks", kind: "AWS::Wickr::Network" },
    OperationMapping { service: "wisdom", operation: "ListAssistants", kind: "AWS::Wisdom::Assistant" },
    OperationMapping { service: "wisdom", operation: "ListKnowledgeBases", kind: "AWS::Wisdom::KnowledgeBase" },
    OperationMapping { service: "workmail", operation: "ListOrganizations", kind: "AWS::ApiOnly::WorkMailOrganization" },
    OperationMapping { service: "workspaces-instances", operation: "ListWorkspaceInstances", kind: "AWS::WorkspacesInstances::WorkspaceInstance" },
    OperationMapping { service: "workspaces-thin-client", operation: "ListDevices", kind: "AWS::ApiOnly::WorkSpacesThinClientDevice" },
];
