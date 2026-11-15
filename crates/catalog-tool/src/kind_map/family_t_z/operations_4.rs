use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "xray", operation: "GetIndexingRules", kind: "AWS::ApiOnly::XRayIndexingRule" },
    OperationMapping { service: "xray", operation: "ListResourcePolicies", kind: "AWS::XRay::ResourcePolicy" },
    OperationMapping { service: "transfer", operation: "ListUsers", kind: "AWS::Transfer::User" },
    OperationMapping { service: "transfer", operation: "ListAgreements", kind: "AWS::Transfer::Agreement" },
    OperationMapping { service: "vpc-lattice", operation: "ListListeners", kind: "AWS::VpcLattice::Listener" },
    OperationMapping { service: "vpc-lattice", operation: "ListServiceNetworkServiceAssociations", kind: "AWS::VpcLattice::ServiceNetworkServiceAssociation" },
    OperationMapping { service: "vpc-lattice", operation: "ListServiceNetworkVpcAssociations", kind: "AWS::VpcLattice::ServiceNetworkVpcAssociation" },
    OperationMapping { service: "verifiedpermissions", operation: "ListIdentitySources", kind: "AWS::VerifiedPermissions::IdentitySource" },
    OperationMapping { service: "verifiedpermissions", operation: "ListPolicyTemplates", kind: "AWS::VerifiedPermissions::PolicyTemplate" },
    OperationMapping { service: "wisdom", operation: "ListAssistantAssociations", kind: "AWS::Wisdom::AssistantAssociation" },
    OperationMapping { service: "transfer", operation: "ListHostKeys", kind: "AWS::Transfer::HostKey" },
    OperationMapping { service: "transcribe", operation: "ListLanguageModels", kind: "AWS::ApiOnly::TranscribeLanguageModel" },
    OperationMapping { service: "transcribe", operation: "ListCallAnalyticsCategories", kind: "AWS::Transcribe::CallAnalyticsCategory" },
    OperationMapping { service: "verifiedpermissions", operation: "ListPolicies", kind: "AWS::VerifiedPermissions::Policy" },
    OperationMapping { service: "vpc-lattice", operation: "ListResourceEndpointAssociations", kind: "AWS::ApiOnly::VpcLatticeResourceEndpointAssociation" },
    OperationMapping { service: "vpc-lattice", operation: "ListServiceNetworkVpcEndpointAssociations", kind: "AWS::ApiOnly::VpcLatticeServiceNetworkVpcEndpointAssociation" },
    OperationMapping { service: "workspaces-web", operation: "ListIdentityProviders", kind: "AWS::WorkSpacesWeb::IdentityProvider" },
    OperationMapping { service: "textract", operation: "GetAdapter", kind: "AWS::Textract::Adapter" },
    OperationMapping { service: "textract", operation: "GetAdapterVersion", kind: "AWS::ApiOnly::TextractAdapterVersion" },
    OperationMapping { service: "timestream-influxdb", operation: "GetDbBackup", kind: "AWS::Timestream::InfluxDBBackup" },
];
