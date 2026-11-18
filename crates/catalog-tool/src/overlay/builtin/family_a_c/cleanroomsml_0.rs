// Pinned source: .local/aws-models/models/cleanroomsml/service/2023-09-06/cleanroomsml-2023-09-06.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelTrainedModel",
            deferred(
                "Model requires membershipIdentifier, trainedModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits a request to cancel the trained model job.",
            ),
        ),
        (
            "CancelTrainedModelInferenceJob",
            deferred(
                "Model requires membershipIdentifier, trainedModelInferenceJobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits a request to cancel a trained model inference job.",
            ),
        ),
        (
            "CreateAudienceModel",
            deferred(
                "Model requires name, trainingDatasetArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines the information necessary to create an audience model. An audience model is a machine learning model that Clean Rooms ML trains to measure similarity between users. Clean Rooms ML manages training and storing the audience…",
            ),
        ),
        (
            "CreateConfiguredAudienceModel",
            deferred(
                "Model requires name, audienceModelArn, outputConfig, sharedAudienceMetrics. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines the information necessary to create a configured audience model.",
            ),
        ),
        (
            "CreateConfiguredModelAlgorithm",
            deferred(
                "Model requires name, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a configured model algorithm using a container image stored in an ECR repository.",
            ),
        ),
        (
            "CreateConfiguredModelAlgorithmAssociation",
            deferred(
                "Model requires membershipIdentifier, configuredModelAlgorithmArn, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a configured model algorithm to a collaboration for use by any member of the collaboration.",
            ),
        ),
        (
            "CreateMLInputChannel",
            deferred(
                "Model requires membershipIdentifier, configuredModelAlgorithmAssociations, inputChannel, name, retentionInDays. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the information to create an ML input channel. An ML input channel is the result of a query that can be used for ML modeling.",
            ),
        ),
        (
            "CreateTrainedModel",
            deferred(
                "Model requires membershipIdentifier, name, configuredModelAlgorithmAssociationArn, resourceConfig, dataChannels. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a trained model from an associated configured model algorithm using data from any member of the collaboration.",
            ),
        ),
        (
            "CreateTrainingDataset",
            deferred(
                "Model requires name, roleArn, trainingData. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines the information necessary to create a training dataset. In Clean Rooms ML, the TrainingDataset is metadata that points to a Glue table, which is read only during AudienceModel creation.",
            ),
        ),
        (
            "DeleteAudienceGenerationJob",
            deferred(
                "Model requires audienceGenerationJobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified audience generation job, and removes all data associated with the job.",
            ),
        ),
        (
            "DeleteAudienceModel",
            deferred(
                "Model requires audienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Specifies an audience model that you want to delete. You can't delete an audience model if there are any configured audience models that depend on the audience model.",
            ),
        ),
        (
            "DeleteConfiguredAudienceModel",
            deferred(
                "Model requires configuredAudienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified configured audience model. You can't delete a configured audience model if there are any lookalike models that use the configured audience model. If you delete a configured audience model, it will be removed…",
            ),
        ),
        (
            "DeleteConfiguredAudienceModelPolicy",
            deferred(
                "Model requires configuredAudienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified configured audience model policy.",
            ),
        ),
        (
            "DeleteConfiguredModelAlgorithm",
            deferred(
                "Model requires configuredModelAlgorithmArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configured model algorithm.",
            ),
        ),
        (
            "DeleteConfiguredModelAlgorithmAssociation",
            deferred(
                "Model requires configuredModelAlgorithmAssociationArn, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configured model algorithm association.",
            ),
        ),
        (
            "DeleteMLConfiguration",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a ML modeling configuration.",
            ),
        ),
        (
            "DeleteMLInputChannelData",
            deferred(
                "Model requires mlInputChannelArn, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the information necessary to delete an ML input channel.",
            ),
        ),
        (
            "DeleteTrainedModelOutput",
            deferred(
                "Model requires trainedModelArn, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the model artifacts stored by the service.",
            ),
        ),
        (
            "DeleteTrainingDataset",
            deferred(
                "Model requires trainingDatasetArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Specifies a training dataset that you want to delete. You can't delete a training dataset if there are any audience models that depend on the training dataset. In Clean Rooms ML, the TrainingDataset is metadata that points to a…",
            ),
        ),
        (
            "GetAudienceGenerationJob",
            deferred(
                "Model requires audienceGenerationJobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an audience generation job.",
            ),
        ),
        (
            "GetAudienceModel",
            deferred(
                "Model requires audienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an audience model",
            ),
        ),
        (
            "GetCollaborationConfiguredModelAlgorithmAssociation",
            deferred(
                "Model requires configuredModelAlgorithmAssociationArn, collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the configured model algorithm association in a collaboration.",
            ),
        ),
        (
            "GetCollaborationMLInputChannel",
            deferred(
                "Model requires mlInputChannelArn, collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specific ML input channel in a collaboration.",
            ),
        ),
        (
            "GetCollaborationTrainedModel",
            deferred(
                "Model requires trainedModelArn, collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a trained model in a collaboration.",
            ),
        ),
        (
            "GetConfiguredAudienceModel",
            deferred(
                "Model requires configuredAudienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specified configured audience model.",
            ),
        ),
        (
            "GetConfiguredAudienceModelPolicy",
            deferred(
                "Model requires configuredAudienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a configured audience model policy.",
            ),
        ),
        (
            "GetConfiguredModelAlgorithm",
            deferred(
                "Model requires configuredModelAlgorithmArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a configured model algorithm.",
            ),
        ),
        (
            "GetConfiguredModelAlgorithmAssociation",
            deferred(
                "Model requires configuredModelAlgorithmAssociationArn, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a configured model algorithm association.",
            ),
        ),
        (
            "GetMLConfiguration",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specific ML configuration.",
            ),
        ),
        (
            "GetMLInputChannel",
            deferred(
                "Model requires mlInputChannelArn, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an ML input channel.",
            ),
        ),
        (
            "GetTrainedModel",
            deferred(
                "Model requires trainedModelArn, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a trained model.",
            ),
        ),
        (
            "GetTrainedModelInferenceJob",
            deferred(
                "Model requires membershipIdentifier, trainedModelInferenceJobArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a trained model inference job.",
            ),
        ),
        (
            "GetTrainingDataset",
            deferred(
                "Model requires trainingDatasetArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a training dataset.",
            ),
        ),
        (
            "ListAudienceExportJobs",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is audienceExportJobs; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of the audience export jobs.",
            ),
        ),
        // Model: Returns a list of audience generation jobs.
        (
            "ListAudienceGenerationJobs",
            OperationOverlay {
                queryable: vec![path("audienceGenerationJobArn")?],
                ..read(
                    "audienceGenerationJobs",
                    Identity::Arn(path("audienceGenerationJobArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns a list of audience models.
        (
            "ListAudienceModels",
            OperationOverlay {
                queryable: vec![path("audienceModelArn")?],
                ..read(
                    "audienceModels",
                    Identity::Arn(path("audienceModelArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListCollaborationConfiguredModelAlgorithmAssociations",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the configured model algorithm associations in a collaboration.",
            ),
        ),
        (
            "ListCollaborationMLInputChannels",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the ML input channels in a collaboration.",
            ),
        ),
        (
            "ListCollaborationTrainedModelExportJobs",
            deferred(
                "Model requires collaborationIdentifier, trainedModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the export jobs for a trained model in a collaboration.",
            ),
        ),
        (
            "ListCollaborationTrainedModelInferenceJobs",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of trained model inference jobs in a specified collaboration.",
            ),
        ),
        (
            "ListCollaborationTrainedModels",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the trained models in a collaboration.",
            ),
        ),
        // Model: Returns a list of the configured audience models.
        (
            "ListConfiguredAudienceModels",
            OperationOverlay {
                queryable: vec![path("configuredAudienceModelArn")?],
                ..read(
                    "configuredAudienceModels",
                    Identity::Arn(path("configuredAudienceModelArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListConfiguredModelAlgorithmAssociations",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of configured model algorithm associations.",
            ),
        ),
        // Model: Returns a list of configured model algorithms.
        (
            "ListConfiguredModelAlgorithms",
            OperationOverlay {
                queryable: vec![path("configuredModelAlgorithmArn")?],
                ..read(
                    "configuredModelAlgorithms",
                    Identity::Arn(path("configuredModelAlgorithmArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListMLInputChannels",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of ML input channels.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags for a provided resource.",
            ),
        ),
        (
            "ListTrainedModelInferenceJobs",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of trained model inference jobs that match the request parameters.",
            ),
        ),
        (
            "ListTrainedModelVersions",
            deferred(
                "Model requires membershipIdentifier, trainedModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of trained model versions for a specified trained model. This operation allows you to view all versions of a trained model, including information about their status and creation details. You can use this to track…",
            ),
        ),
        (
            "ListTrainedModels",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of trained models.",
            ),
        ),
        // Model: Returns a list of training datasets.
        (
            "ListTrainingDatasets",
            OperationOverlay {
                queryable: vec![path("trainingDatasetArn")?],
                ..read(
                    "trainingDatasets",
                    Identity::Arn(path("trainingDatasetArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "PutConfiguredAudienceModelPolicy",
            deferred(
                "Model requires configuredAudienceModelArn, configuredAudienceModelPolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create or update the resource policy for a configured audience model.",
            ),
        ),
        (
            "PutMLConfiguration",
            deferred(
                "Model requires membershipIdentifier, defaultOutputLocation. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns information about an ML configuration.",
            ),
        ),
        (
            "StartAudienceExportJob",
            deferred(
                "Model requires name, audienceGenerationJobArn, audienceSize. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Export an audience of a specified size after you have generated an audience.",
            ),
        ),
        (
            "StartAudienceGenerationJob",
            deferred(
                "Model requires name, configuredAudienceModelArn, seedAudience. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Information necessary to start the audience generation job.",
            ),
        ),
        (
            "StartTrainedModelExportJob",
            deferred(
                "Model requires name, trainedModelArn, membershipIdentifier, outputConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the information necessary to start a trained model export job.",
            ),
        ),
        (
            "StartTrainedModelInferenceJob",
            deferred(
                "Model requires membershipIdentifier, name, trainedModelArn, resourceConfig, outputConfiguration, dataSource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines the information necessary to begin a trained model inference job.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds metadata tags to a specified resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes metadata tags from a specified resource.",
            ),
        ),
        (
            "UpdateConfiguredAudienceModel",
            deferred(
                "Model requires configuredAudienceModelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the information necessary to update a configured audience model. Updates that impact audience generation jobs take effect when a new job starts, but do not impact currently running jobs.",
            ),
        ),
    ])
}
