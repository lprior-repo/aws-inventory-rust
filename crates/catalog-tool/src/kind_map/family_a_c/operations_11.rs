use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "chime-sdk-voice", operation: "GetVoiceConnector", kind: "AWS::Chime::VoiceConnector" },
    OperationMapping { service: "chime-sdk-voice", operation: "GetVoiceConnectorGroup", kind: "AWS::ApiOnly::ChimeVoiceConnectorGroup" },
    OperationMapping { service: "chime-sdk-voice", operation: "ListSipMediaApplications", kind: "AWS::Chime::SipMediaApplication" },
    OperationMapping { service: "chime-sdk-voice", operation: "ListSipRules", kind: "AWS::ApiOnly::ChimeSipRule" },
    OperationMapping { service: "chime-sdk-voice", operation: "ListVoiceConnectorGroups", kind: "AWS::ApiOnly::ChimeVoiceConnectorGroup" },
    OperationMapping { service: "chime-sdk-voice", operation: "ListVoiceConnectors", kind: "AWS::Chime::VoiceConnector" },
    OperationMapping { service: "chime", operation: "ListAccounts", kind: "AWS::ApiOnly::ChimeAccount" },
    OperationMapping { service: "cleanrooms", operation: "GetCollaboration", kind: "AWS::CleanRooms::Collaboration" },
    OperationMapping { service: "cleanrooms", operation: "GetConfiguredTable", kind: "AWS::CleanRooms::ConfiguredTable" },
    OperationMapping { service: "cleanrooms", operation: "GetMembership", kind: "AWS::CleanRooms::Membership" },
    OperationMapping { service: "cleanrooms", operation: "ListCollaborations", kind: "AWS::CleanRooms::Collaboration" },
    OperationMapping { service: "cleanrooms", operation: "ListConfiguredTables", kind: "AWS::CleanRooms::ConfiguredTable" },
    OperationMapping { service: "cleanrooms", operation: "ListMemberships", kind: "AWS::CleanRooms::Membership" },
    OperationMapping { service: "cleanroomsml", operation: "ListAudienceGenerationJobs", kind: "AWS::ApiOnly::CleanRoomsMLAudienceGenerationJob" },
    OperationMapping { service: "cleanroomsml", operation: "ListAudienceModels", kind: "AWS::ApiOnly::CleanRoomsMLAudienceModel" },
    OperationMapping { service: "cleanroomsml", operation: "ListConfiguredAudienceModels", kind: "AWS::ApiOnly::CleanRoomsMLConfiguredAudienceModel" },
    OperationMapping { service: "cleanroomsml", operation: "ListConfiguredModelAlgorithms", kind: "AWS::ApiOnly::CleanRoomsMLConfiguredModelAlgorithm" },
    OperationMapping { service: "cleanroomsml", operation: "ListTrainingDatasets", kind: "AWS::ApiOnly::CleanRoomsMLTrainingDataset" },
    OperationMapping { service: "cloud9", operation: "ListEnvironments", kind: "AWS::ApiOnly::Cloud9Environment" },
    OperationMapping { service: "clouddirectory", operation: "ListDevelopmentSchemaArns", kind: "AWS::ApiOnly::CloudDirectorySchema" },
];
