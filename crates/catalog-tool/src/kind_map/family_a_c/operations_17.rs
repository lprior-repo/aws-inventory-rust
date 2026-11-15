use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "comprehend", operation: "ListDominantLanguageDetectionJobs", kind: "AWS::ApiOnly::ComprehendDominantLanguageDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListEndpoints", kind: "AWS::ApiOnly::ComprehendEndpoint" },
    OperationMapping { service: "comprehend", operation: "ListEntitiesDetectionJobs", kind: "AWS::ApiOnly::ComprehendEntitiesDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListEntityRecognizers", kind: "AWS::Comprehend::EntityRecognizer" },
    OperationMapping { service: "comprehend", operation: "ListEventsDetectionJobs", kind: "AWS::ApiOnly::ComprehendEventsDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListFlywheels", kind: "AWS::Comprehend::Flywheel" },
    OperationMapping { service: "comprehend", operation: "ListKeyPhrasesDetectionJobs", kind: "AWS::ApiOnly::ComprehendKeyPhrasesDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListPiiEntitiesDetectionJobs", kind: "AWS::ApiOnly::ComprehendPiiEntitiesDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListSentimentDetectionJobs", kind: "AWS::ApiOnly::ComprehendSentimentDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListTargetedSentimentDetectionJobs", kind: "AWS::ApiOnly::ComprehendTargetedSentimentDetectionJob" },
    OperationMapping { service: "comprehend", operation: "ListTopicsDetectionJobs", kind: "AWS::ApiOnly::ComprehendTopicsDetectionJob" },
    OperationMapping { service: "comprehendmedical", operation: "ListEntitiesDetectionV2Jobs", kind: "AWS::ApiOnly::ComprehendMedicalEntitiesDetectionJob" },
    OperationMapping { service: "comprehendmedical", operation: "ListICD10CMInferenceJobs", kind: "AWS::ApiOnly::ComprehendMedicalICD10CMInferenceJob" },
    OperationMapping { service: "comprehendmedical", operation: "ListPHIDetectionJobs", kind: "AWS::ApiOnly::ComprehendMedicalPHIDetectionJob" },
    OperationMapping { service: "comprehendmedical", operation: "ListRxNormInferenceJobs", kind: "AWS::ApiOnly::ComprehendMedicalRxNormInferenceJob" },
    OperationMapping { service: "comprehendmedical", operation: "ListSNOMEDCTInferenceJobs", kind: "AWS::ApiOnly::ComprehendMedicalSNOMEDCTInferenceJob" },
    OperationMapping { service: "compute-optimizer-automation", operation: "ListAutomationEvents", kind: "AWS::ApiOnly::ComputeOptimizerAutomationAutomationEvent" },
    OperationMapping { service: "compute-optimizer-automation", operation: "ListAutomationRules", kind: "AWS::ApiOnly::ComputeOptimizerAutomationAutomationRule" },
    OperationMapping { service: "compute-optimizer", operation: "DescribeRecommendationExportJobs", kind: "AWS::ApiOnly::ComputeOptimizerRecommendationExportJob" },
    OperationMapping { service: "config-service", operation: "DescribeAggregationAuthorizations", kind: "AWS::Config::AggregationAuthorization" },
];
