use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "qbusiness", operation: "ListApplications", kind: "AWS::QBusiness::Application" },
    OperationMapping { service: "qbusiness", operation: "ListRetrievers", kind: "AWS::QBusiness::Retriever" },
    OperationMapping { service: "qconnect", operation: "ListAssistants", kind: "AWS::Wisdom::Assistant" },
    OperationMapping { service: "qconnect", operation: "ListKnowledgeBases", kind: "AWS::Wisdom::KnowledgeBase" },
    OperationMapping { service: "qconnect", operation: "ListAssistantAssociations", kind: "AWS::Wisdom::AssistantAssociation" },
    OperationMapping { service: "quicksight", operation: "ListAnalyses", kind: "AWS::QuickSight::Analysis" },
    OperationMapping { service: "quicksight", operation: "ListDashboards", kind: "AWS::QuickSight::Dashboard" },
    OperationMapping { service: "quicksight", operation: "ListDataSets", kind: "AWS::QuickSight::DataSet" },
    OperationMapping { service: "quicksight", operation: "ListDataSources", kind: "AWS::QuickSight::DataSource" },
    OperationMapping { service: "quicksight", operation: "ListFolders", kind: "AWS::QuickSight::Folder" },
    OperationMapping { service: "quicksight", operation: "ListThemes", kind: "AWS::QuickSight::Theme" },
    OperationMapping { service: "quicksight", operation: "ListNamespaces", kind: "AWS::ApiOnly::QuickSightNamespace" },
    OperationMapping { service: "quicksight", operation: "ListVPCConnections", kind: "AWS::QuickSight::VPCConnection" },
    OperationMapping { service: "quicksight", operation: "ListCustomPermissions", kind: "AWS::QuickSight::CustomPermissions" },
    OperationMapping { service: "quicksight", operation: "ListApprovalPolicies", kind: "AWS::QuickSight::ApprovalPolicy" },
    OperationMapping { service: "ram", operation: "GetResourceShares", kind: "AWS::RAM::ResourceShare" },
    OperationMapping { service: "ram", operation: "GetResourceShareInvitations", kind: "AWS::ApiOnly::RAMResourceShareInvitation" },
    OperationMapping { service: "ram", operation: "ListPermissionAssociations", kind: "AWS::ApiOnly::RAMPermissionAssociation" },
    OperationMapping { service: "ram", operation: "ListSourceAssociations", kind: "AWS::ApiOnly::RAMSourceAssociation" },
    OperationMapping { service: "ram", operation: "ListReplacePermissionAssociationsWork", kind: "AWS::ApiOnly::RAMPermissionReplacement" },
];
