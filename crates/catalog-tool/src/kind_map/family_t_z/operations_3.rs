use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "workspaces-web",
        operation: "ListBrowserSettings",
        kind: "AWS::WorkSpacesWeb::BrowserSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListDataProtectionSettings",
        kind: "AWS::WorkSpacesWeb::DataProtectionSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListIpAccessSettings",
        kind: "AWS::WorkSpacesWeb::IpAccessSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListNetworkSettings",
        kind: "AWS::WorkSpacesWeb::NetworkSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListPortals",
        kind: "AWS::WorkSpacesWeb::Portal",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListSessionLoggers",
        kind: "AWS::WorkSpacesWeb::SessionLogger",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListTrustStores",
        kind: "AWS::WorkSpacesWeb::TrustStore",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListUserAccessLoggingSettings",
        kind: "AWS::WorkSpacesWeb::UserAccessLoggingSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "ListUserSettings",
        kind: "AWS::WorkSpacesWeb::UserSettings",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeApplications",
        kind: "AWS::ApiOnly::WorkSpacesApplication",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeConnectionAliases",
        kind: "AWS::WorkSpaces::ConnectionAlias",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeIpGroups",
        kind: "AWS::WorkSpaces::WorkspaceIpGroup",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeWorkspaceBundles",
        kind: "AWS::ApiOnly::WorkSpacesBundle",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeWorkspaceDirectories",
        kind: "AWS::WorkSpaces::Directory",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeWorkspaceImages",
        kind: "AWS::ApiOnly::WorkSpacesImage",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeWorkspaces",
        kind: "AWS::WorkSpaces::Workspace",
    },
    OperationMapping {
        service: "workspaces",
        operation: "DescribeWorkspacesPools",
        kind: "AWS::WorkSpaces::WorkspacesPool",
    },
    OperationMapping {
        service: "workspaces",
        operation: "ListAccountLinks",
        kind: "AWS::ApiOnly::WorkSpacesAccountLink",
    },
    OperationMapping {
        service: "xray",
        operation: "GetGroups",
        kind: "AWS::XRay::Group",
    },
    OperationMapping {
        service: "xray",
        operation: "GetSamplingRules",
        kind: "AWS::XRay::SamplingRule",
    },
];
