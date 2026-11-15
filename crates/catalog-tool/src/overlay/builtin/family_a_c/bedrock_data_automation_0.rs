// Pinned source: .local/aws-models/models/bedrock-data-automation/service/2023-07-26/bedrock-data-automation-2023-07-26.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CopyBlueprintStage", deferred("Model requires blueprintArn, sourceStage, targetStage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Copies a Blueprint from one stage to another")),
        ("CreateBlueprint", deferred("Model requires blueprintName, type, schema. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Bedrock Data Automation Blueprint")),
        ("CreateBlueprintVersion", deferred("Model requires blueprintArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new version of an existing Amazon Bedrock Data Automation Blueprint")),
        ("CreateDataAutomationLibrary", deferred("Model requires libraryName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Bedrock Data Automation Library")),
        ("CreateDataAutomationProject", deferred("Model requires projectName, standardOutputConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Bedrock Data Automation Project")),
        ("DeleteBlueprint", deferred("Model requires blueprintArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing Amazon Bedrock Data Automation Blueprint")),
        ("DeleteDataAutomationLibrary", deferred("Model requires libraryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing Amazon Bedrock Data Automation Library")),
        ("DeleteDataAutomationProject", deferred("Model requires projectArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing Amazon Bedrock Data Automation Project")),
        ("GetBlueprint", deferred("Model requires blueprintArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an existing Amazon Bedrock Data Automation Blueprint")),
        ("GetBlueprintOptimizationStatus", deferred("Model requires invocationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: API used to get blueprint optimization status.")),
        ("GetDataAutomationLibrary", deferred("Model requires libraryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an existing Amazon Bedrock Data Automation Library")),
        ("GetDataAutomationLibraryEntity", deferred("Model requires libraryArn, entityType, entityId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an existing entity based on entity type from the library")),
        ("GetDataAutomationLibraryIngestionJob", deferred("Model requires libraryArn, jobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: API used to get status of data automation library ingestion job")),
        ("GetDataAutomationProject", deferred("Model requires projectArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an existing Amazon Bedrock Data Automation Project")),
        ("InvokeBlueprintOptimizationAsync", deferred("Model requires blueprint, samples, outputConfiguration, dataAutomationProfileArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Invoke an async job to perform Blueprint Optimization")),
        ("InvokeDataAutomationLibraryIngestionJob", deferred("Model requires libraryArn, inputConfiguration, entityType, operationType, outputConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Async API: Invoke data automation library ingestion job")),
        // Model: Lists all existing Amazon Bedrock Data Automation Blueprints
        ("ListBlueprints", OperationOverlay {
            fixed: [(path("resourceOwner")?, FixedValue::String("ACCOUNT".to_owned()))].into_iter().collect(),
            queryable: vec![path("blueprintArn")?],
            ..read("blueprints", Identity::Arn(path("blueprintArn")?), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        // Model: Lists all existing Amazon Bedrock Data Automation Libraries
        ("ListDataAutomationLibraries", OperationOverlay {
            queryable: vec![path("libraryArn")?],
            ..read("libraries", Identity::Arn(path("libraryArn")?), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListDataAutomationLibraryEntities", deferred("Model requires libraryArn, entityType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all stored entities in the library")),
        ("ListDataAutomationLibraryIngestionJobs", deferred("Model requires libraryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all data automation library ingestion jobs")),
        // Model: Lists all existing Amazon Bedrock Data Automation Projects
        ("ListDataAutomationProjects", OperationOverlay {
            fixed: [(path("resourceOwner")?, FixedValue::String("ACCOUNT".to_owned()))].into_iter().collect(),
            queryable: vec![path("projectArn")?],
            ..read("projects", Identity::Arn(path("projectArn")?), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListTagsForResource", deferred("Model requires resourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List tags for an Amazon Bedrock Data Automation resource")),
        ("TagResource", deferred("Model requires resourceARN, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tag an Amazon Bedrock Data Automation resource")),
        ("UntagResource", deferred("Model requires resourceARN, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Untag an Amazon Bedrock Data Automation resource")),
        ("UpdateBlueprint", deferred("Model requires blueprintArn, schema. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Amazon Bedrock Data Automation Blueprint")),
        ("UpdateDataAutomationLibrary", deferred("Model requires libraryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Amazon Bedrock Data Automation Library")),
        ("UpdateDataAutomationProject", deferred("Model requires projectArn, standardOutputConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Amazon Bedrock Data Automation Project")),
    ])
}
