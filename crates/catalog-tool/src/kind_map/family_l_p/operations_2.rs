use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "lookoutequipment", operation: "ListLabelGroups", kind: "AWS::ApiOnly::LookoutEquipmentLabelGroup" },
    OperationMapping { service: "lookoutequipment", operation: "ListRetrainingSchedulers", kind: "AWS::ApiOnly::LookoutEquipmentRetrainingScheduler" },
    OperationMapping { service: "m2", operation: "ListApplications", kind: "AWS::M2::Application" },
    OperationMapping { service: "m2", operation: "ListEnvironments", kind: "AWS::M2::Environment" },
    OperationMapping { service: "m2", operation: "GetEnvironment", kind: "AWS::M2::Environment" },
    OperationMapping { service: "machine-learning", operation: "DescribeMLModels", kind: "AWS::ApiOnly::MachineLearningModel" },
    OperationMapping { service: "machine-learning", operation: "DescribeEvaluations", kind: "AWS::ApiOnly::MachineLearningEvaluation" },
    OperationMapping { service: "machine-learning", operation: "DescribeBatchPredictions", kind: "AWS::ApiOnly::MachineLearningBatchPrediction" },
    OperationMapping { service: "macie2", operation: "ListAllowLists", kind: "AWS::Macie::AllowList" },
    OperationMapping { service: "macie2", operation: "ListClassificationJobs", kind: "AWS::ApiOnly::MacieClassificationJob" },
    OperationMapping { service: "macie2", operation: "ListCustomDataIdentifiers", kind: "AWS::Macie::CustomDataIdentifier" },
    OperationMapping { service: "macie2", operation: "ListFindingsFilters", kind: "AWS::Macie::FindingsFilter" },
    OperationMapping { service: "mailmanager", operation: "ListAddonInstances", kind: "AWS::SES::MailManagerAddonInstance" },
    OperationMapping { service: "mailmanager", operation: "ListAddonSubscriptions", kind: "AWS::SES::MailManagerAddonSubscription" },
    OperationMapping { service: "mailmanager", operation: "ListAddressLists", kind: "AWS::SES::MailManagerAddressList" },
    OperationMapping { service: "mailmanager", operation: "ListArchives", kind: "AWS::SES::MailManagerArchive" },
    OperationMapping { service: "mailmanager", operation: "ListIngressPoints", kind: "AWS::SES::MailManagerIngressPoint" },
    OperationMapping { service: "mailmanager", operation: "ListRelays", kind: "AWS::SES::MailManagerRelay" },
    OperationMapping { service: "mailmanager", operation: "ListRuleSets", kind: "AWS::SES::MailManagerRuleSet" },
    OperationMapping { service: "mailmanager", operation: "ListTrafficPolicies", kind: "AWS::SES::MailManagerTrafficPolicy" },
];
