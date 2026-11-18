use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "network-security-manager",
        operation: "ListDeployments",
        kind: "AWS::NetworkSecurityManager::Deployment",
    },
    OperationMapping {
        service: "networkflowmonitor",
        operation: "ListMonitors",
        kind: "AWS::NetworkFlowMonitor::Monitor",
    },
    OperationMapping {
        service: "networkflowmonitor",
        operation: "ListScopes",
        kind: "AWS::NetworkFlowMonitor::Scope",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "DescribeGlobalNetworks",
        kind: "AWS::NetworkManager::GlobalNetwork",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "ListAttachments",
        kind: "AWS::ApiOnly::NetworkManagerAttachment",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "ListConnectPeers",
        kind: "AWS::NetworkManager::ConnectPeer",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "ListCoreNetworks",
        kind: "AWS::NetworkManager::CoreNetwork",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "ListPeerings",
        kind: "AWS::ApiOnly::NetworkManagerPeering",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "GetSites",
        kind: "AWS::NetworkManager::Site",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "GetDevices",
        kind: "AWS::NetworkManager::Device",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "GetLinks",
        kind: "AWS::NetworkManager::Link",
    },
    OperationMapping {
        service: "networkmanager",
        operation: "GetConnections",
        kind: "AWS::ApiOnly::NetworkManagerConnection",
    },
    OperationMapping {
        service: "networkmonitor",
        operation: "ListMonitors",
        kind: "AWS::NetworkMonitor::Monitor",
    },
    OperationMapping {
        service: "notifications",
        operation: "ListNotificationConfigurations",
        kind: "AWS::Notifications::NotificationConfiguration",
    },
    OperationMapping {
        service: "notifications",
        operation: "ListManagedNotificationConfigurations",
        kind: "AWS::ApiOnly::ManagedNotificationConfiguration",
    },
    OperationMapping {
        service: "notifications",
        operation: "ListNotificationHubs",
        kind: "AWS::Notifications::NotificationHub",
    },
    OperationMapping {
        service: "notificationscontacts",
        operation: "ListEmailContacts",
        kind: "AWS::NotificationsContacts::EmailContact",
    },
    OperationMapping {
        service: "nova-act",
        operation: "ListWorkflowDefinitions",
        kind: "AWS::NovaAct::WorkflowDefinition",
    },
    OperationMapping {
        service: "oam",
        operation: "ListLinks",
        kind: "AWS::Oam::Link",
    },
    OperationMapping {
        service: "oam",
        operation: "ListSinks",
        kind: "AWS::Oam::Sink",
    },
];
