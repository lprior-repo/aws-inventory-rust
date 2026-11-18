use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "resiliencehub",
        operation: "ListApps",
        kind: "AWS::ResilienceHub::App",
    },
    OperationMapping {
        service: "resiliencehub",
        operation: "ListResiliencyPolicies",
        kind: "AWS::ResilienceHub::ResiliencyPolicy",
    },
    OperationMapping {
        service: "resiliencehub",
        operation: "ListAppAssessments",
        kind: "AWS::ApiOnly::ResilienceHubAppAssessment",
    },
    OperationMapping {
        service: "resiliencehub",
        operation: "ListRecommendationTemplates",
        kind: "AWS::ApiOnly::ResilienceHubRecommendationTemplate",
    },
    OperationMapping {
        service: "resiliencehubv2",
        operation: "ListSystems",
        kind: "AWS::ApiOnly::ResilienceHubSystem",
    },
    OperationMapping {
        service: "resiliencehubv2",
        operation: "ListServices",
        kind: "AWS::ApiOnly::ResilienceHubService",
    },
    OperationMapping {
        service: "resiliencehubv2",
        operation: "ListPolicies",
        kind: "AWS::ApiOnly::ResilienceHubPolicy",
    },
    OperationMapping {
        service: "resiliencehubv2",
        operation: "ListDependencies",
        kind: "AWS::ApiOnly::ResilienceHubDependency",
    },
    OperationMapping {
        service: "resource-explorer-2",
        operation: "ListViews",
        kind: "AWS::ResourceExplorer2::View",
    },
    OperationMapping {
        service: "resource-explorer-2",
        operation: "GetIndex",
        kind: "AWS::ResourceExplorer2::Index",
    },
    OperationMapping {
        service: "resource-explorer-2",
        operation: "ListStreamingAccessForServices",
        kind: "AWS::ApiOnly::ResourceExplorerStreamingAccess",
    },
    OperationMapping {
        service: "resource-groups",
        operation: "ListGroups",
        kind: "AWS::ResourceGroups::Group",
    },
    OperationMapping {
        service: "resource-groups",
        operation: "ListTagSyncTasks",
        kind: "AWS::ApiOnly::ResourceGroupTagSyncTask",
    },
    OperationMapping {
        service: "rolesanywhere",
        operation: "ListCrls",
        kind: "AWS::RolesAnywhere::CRL",
    },
    OperationMapping {
        service: "rolesanywhere",
        operation: "ListProfiles",
        kind: "AWS::RolesAnywhere::Profile",
    },
    OperationMapping {
        service: "rolesanywhere",
        operation: "ListTrustAnchors",
        kind: "AWS::RolesAnywhere::TrustAnchor",
    },
    OperationMapping {
        service: "rolesanywhere",
        operation: "ListSubjects",
        kind: "AWS::ApiOnly::RolesAnywhereSubject",
    },
    OperationMapping {
        service: "route53-recovery-control-config",
        operation: "ListClusters",
        kind: "AWS::Route53RecoveryControl::Cluster",
    },
    OperationMapping {
        service: "route53-recovery-control-config",
        operation: "ListControlPanels",
        kind: "AWS::Route53RecoveryControl::ControlPanel",
    },
    OperationMapping {
        service: "route53-recovery-control-config",
        operation: "ListRoutingControls",
        kind: "AWS::Route53RecoveryControl::RoutingControl",
    },
];
