// Pinned source: .local/aws-models/models/comprehendmedical/service/2018-10-30/comprehendmedical-2018-10-30.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("DescribeEntitiesDetectionV2Job", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the properties associated with a medical entities detection job. Use this operation to get the status of a detection job.")),
        ("DescribeICD10CMInferenceJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the properties associated with an InferICD10CM job. Use this operation to get the status of an inference job.")),
        ("DescribePHIDetectionJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the properties associated with a protected health information (PHI) detection job. Use this operation to get the status of a detection job.")),
        ("DescribeRxNormInferenceJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the properties associated with an InferRxNorm job. Use this operation to get the status of an inference job.")),
        ("DescribeSNOMEDCTInferenceJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the properties associated with an InferSNOMEDCT job. Use this operation to get the status of an inference job.")),
        ("DetectEntities", deferred("Model requires Text. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The DetectEntities operation is deprecated. You should use the DetectEntitiesV2 operation instead. Inspects the clinical text for a variety of medical entities and returns specific information about them such as entity category,…")),
        ("DetectEntitiesV2", deferred("Model requires Text. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Inspects the clinical text for a variety of medical entities and returns specific information about them such as entity category, location, and confidence score on that information. Amazon Comprehend Medical only detects medical…")),
        ("DetectPHI", deferred("Model requires Text. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Inspects the clinical text for protected health information (PHI) entities and returns the entity category, location, and confidence score for each entity. Amazon Comprehend Medical only detects entities in English language texts.")),
        ("InferICD10CM", deferred("Model requires Text. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: InferICD10CM detects medical conditions as entities listed in a patient record and links those entities to normalized concept identifiers in the ICD-10-CM knowledge base from the Centers for Disease Control. Amazon Comprehend…")),
        ("InferRxNorm", deferred("Model requires Text. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: InferRxNorm detects medications as entities listed in a patient record and links to the normalized concept identifiers in the RxNorm database from the National Library of Medicine. Amazon Comprehend Medical only detects medical…")),
        ("InferSNOMEDCT", deferred("Model requires Text. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: InferSNOMEDCT detects possible medical concepts as entities and links them to codes from the Systematized Nomenclature of Medicine, Clinical Terms (SNOMED-CT) ontology")),
        // Model: Gets a list of medical entity detection jobs that you have submitted.
        ("ListEntitiesDetectionV2Jobs", OperationOverlay {
            queryable: vec![path("JobId")?],
            ..read("ComprehendMedicalAsyncJobPropertiesList", Identity::Key(vec![path("JobId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Gets a list of InferICD10CM jobs that you have submitted.
        ("ListICD10CMInferenceJobs", OperationOverlay {
            queryable: vec![path("JobId")?],
            ..read("ComprehendMedicalAsyncJobPropertiesList", Identity::Key(vec![path("JobId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Gets a list of protected health information (PHI) detection jobs you have submitted.
        ("ListPHIDetectionJobs", OperationOverlay {
            queryable: vec![path("JobId")?],
            ..read("ComprehendMedicalAsyncJobPropertiesList", Identity::Key(vec![path("JobId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Gets a list of InferRxNorm jobs that you have submitted.
        ("ListRxNormInferenceJobs", OperationOverlay {
            queryable: vec![path("JobId")?],
            ..read("ComprehendMedicalAsyncJobPropertiesList", Identity::Key(vec![path("JobId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Gets a list of InferSNOMEDCT jobs a user has submitted.
        ("ListSNOMEDCTInferenceJobs", OperationOverlay {
            queryable: vec![path("JobId")?],
            ..read("ComprehendMedicalAsyncJobPropertiesList", Identity::Key(vec![path("JobId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("StartEntitiesDetectionV2Job", deferred("Model requires InputDataConfig, OutputDataConfig, DataAccessRoleArn, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an asynchronous medical entity detection job for a collection of documents. Use the DescribeEntitiesDetectionV2Job operation to track the status of a job.")),
        ("StartICD10CMInferenceJob", deferred("Model requires InputDataConfig, OutputDataConfig, DataAccessRoleArn, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an asynchronous job to detect medical conditions and link them to the ICD-10-CM ontology. Use the DescribeICD10CMInferenceJob operation to track the status of a job.")),
        ("StartPHIDetectionJob", deferred("Model requires InputDataConfig, OutputDataConfig, DataAccessRoleArn, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an asynchronous job to detect protected health information (PHI). Use the DescribePHIDetectionJob operation to track the status of a job.")),
        ("StartRxNormInferenceJob", deferred("Model requires InputDataConfig, OutputDataConfig, DataAccessRoleArn, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an asynchronous job to detect medication entities and link them to the RxNorm ontology. Use the DescribeRxNormInferenceJob operation to track the status of a job.")),
        ("StartSNOMEDCTInferenceJob", deferred("Model requires InputDataConfig, OutputDataConfig, DataAccessRoleArn, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an asynchronous job to detect medical concepts and link them to the SNOMED-CT ontology. Use the DescribeSNOMEDCTInferenceJob operation to track the status of a job.")),
        ("StopEntitiesDetectionV2Job", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a medical entities detection job in progress.")),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("StopICD10CMInferenceJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an InferICD10CM inference job in progress.")),
        ("StopPHIDetectionJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a protected health information (PHI) detection job in progress.")),
        ("StopRxNormInferenceJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an InferRxNorm inference job in progress.")),
        ("StopSNOMEDCTInferenceJob", deferred("Model requires JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an InferSNOMEDCT inference job in progress.")),
    ])
}
