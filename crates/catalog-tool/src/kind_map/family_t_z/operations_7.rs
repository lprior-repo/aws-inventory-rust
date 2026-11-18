use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "verifiedpermissions",
        operation: "GetPolicyStore",
        kind: "AWS::VerifiedPermissions::PolicyStore",
    },
    OperationMapping {
        service: "verifiedpermissions",
        operation: "GetPolicyStoreAlias",
        kind: "AWS::VerifiedPermissions::PolicyStoreAlias",
    },
    OperationMapping {
        service: "verifiedpermissions",
        operation: "GetIdentitySource",
        kind: "AWS::VerifiedPermissions::IdentitySource",
    },
    OperationMapping {
        service: "wickr",
        operation: "GetNetwork",
        kind: "AWS::Wickr::Network",
    },
    OperationMapping {
        service: "wisdom",
        operation: "GetAssistant",
        kind: "AWS::Wisdom::Assistant",
    },
    OperationMapping {
        service: "wisdom",
        operation: "GetKnowledgeBase",
        kind: "AWS::Wisdom::KnowledgeBase",
    },
    OperationMapping {
        service: "workspaces-instances",
        operation: "GetWorkspaceInstance",
        kind: "AWS::WorkspacesInstances::WorkspaceInstance",
    },
    OperationMapping {
        service: "workspaces-thin-client",
        operation: "GetDevice",
        kind: "AWS::ApiOnly::WorkSpacesThinClientDevice",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetBrowserSettings",
        kind: "AWS::WorkSpacesWeb::BrowserSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetDataProtectionSettings",
        kind: "AWS::WorkSpacesWeb::DataProtectionSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetIpAccessSettings",
        kind: "AWS::WorkSpacesWeb::IpAccessSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetNetworkSettings",
        kind: "AWS::WorkSpacesWeb::NetworkSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetPortal",
        kind: "AWS::WorkSpacesWeb::Portal",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetSessionLogger",
        kind: "AWS::WorkSpacesWeb::SessionLogger",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetTrustStore",
        kind: "AWS::WorkSpacesWeb::TrustStore",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetUserAccessLoggingSettings",
        kind: "AWS::WorkSpacesWeb::UserAccessLoggingSettings",
    },
    OperationMapping {
        service: "workspaces-web",
        operation: "GetUserSettings",
        kind: "AWS::WorkSpacesWeb::UserSettings",
    },
    OperationMapping {
        service: "workmail",
        operation: "DescribeOrganization",
        kind: "AWS::ApiOnly::WorkMailOrganization",
    },
    OperationMapping {
        service: "xray",
        operation: "GetGroup",
        kind: "AWS::XRay::Group",
    },
    OperationMapping {
        service: "wisdom",
        operation: "GetAssistantAssociation",
        kind: "AWS::Wisdom::AssistantAssociation",
    },
];
