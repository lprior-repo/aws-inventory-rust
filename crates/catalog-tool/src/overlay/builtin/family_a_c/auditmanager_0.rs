use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateAssessmentReportEvidenceFolder",
            deferred(
                "Model requires assessmentId, evidenceFolderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates an evidence folder to an assessment report in an Audit Manager assessment.",
            ),
        ),
        (
            "BatchAssociateAssessmentReportEvidence",
            deferred(
                "Model requires assessmentId, evidenceFolderId, evidenceIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a list of evidence to an assessment report in an Audit Manager assessment.",
            ),
        ),
        (
            "BatchCreateDelegationByAssessment",
            deferred(
                "Model requires createDelegationRequests, assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a batch of delegations for an assessment in Audit Manager.",
            ),
        ),
        (
            "BatchDeleteDelegationByAssessment",
            deferred(
                "Model requires delegationIds, assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a batch of delegations for an assessment in Audit Manager.",
            ),
        ),
        (
            "BatchDisassociateAssessmentReportEvidence",
            deferred(
                "Model requires assessmentId, evidenceFolderId, evidenceIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a list of evidence from an assessment report in Audit Manager.",
            ),
        ),
        (
            "BatchImportEvidenceToAssessmentControl",
            deferred(
                "Model requires assessmentId, controlSetId, controlId, manualEvidence. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds one or more pieces of evidence to a control in an Audit Manager assessment. You can import manual evidence from any S3 bucket by specifying the S3 URI of the object. You can also upload a file from your browser, or enter…",
            ),
        ),
        (
            "CreateAssessment",
            deferred(
                "Model requires name, assessmentReportsDestination, scope, roles, frameworkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an assessment in Audit Manager.",
            ),
        ),
        (
            "CreateAssessmentFramework",
            deferred(
                "Model requires name, controlSets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom framework in Audit Manager.",
            ),
        ),
        (
            "CreateAssessmentReport",
            deferred(
                "Model requires name, assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an assessment report for the specified assessment.",
            ),
        ),
        (
            "CreateControl",
            deferred(
                "Model requires name, controlMappingSources. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new custom control in Audit Manager.",
            ),
        ),
        (
            "DeleteAssessment",
            deferred(
                "Model requires assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an assessment in Audit Manager.",
            ),
        ),
        (
            "DeleteAssessmentFramework",
            deferred(
                "Model requires frameworkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom framework in Audit Manager.",
            ),
        ),
        (
            "DeleteAssessmentFrameworkShare",
            deferred(
                "Model requires requestId, requestType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a share request for a custom framework in Audit Manager.",
            ),
        ),
        (
            "DeleteAssessmentReport",
            deferred(
                "Model requires assessmentId, assessmentReportId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an assessment report in Audit Manager. When you run the DeleteAssessmentReport operation, Audit Manager attempts to delete the following data: The specified assessment report that’s stored in your S3 bucket The associated…",
            ),
        ),
        (
            "DeleteControl",
            deferred(
                "Model requires controlId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom control in Audit Manager. When you invoke this operation, the custom control is deleted from any frameworks or assessments that it’s currently part of. As a result, Audit Manager will stop collecting evidence for…",
            ),
        ),
        (
            "DeregisterAccount",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is status; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deregisters an account in Audit Manager. Before you deregister, you can use the UpdateSettings API operation to set your preferred data retention policy. By default, Audit Manager retains your data. If you want to delete your…",
            ),
        ),
        (
            "DeregisterOrganizationAdminAccount",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Removes the specified Amazon Web Services account as a delegated administrator for Audit Manager. When you remove a delegated administrator from your Audit Manager settings, you continue to have access to the evidence that you…",
            ),
        ),
        (
            "DisassociateAssessmentReportEvidenceFolder",
            deferred(
                "Model requires assessmentId, evidenceFolderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates an evidence folder from the specified assessment report in Audit Manager.",
            ),
        ),
        (
            "GetAccountStatus",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is status; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets the registration status of an account in Audit Manager.",
            ),
        ),
        (
            "GetAssessment",
            deferred(
                "Model requires assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a specified assessment.",
            ),
        ),
        (
            "GetAssessmentFramework",
            deferred(
                "Model requires frameworkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a specified framework.",
            ),
        ),
        (
            "GetAssessmentReportUrl",
            deferred(
                "Model requires assessmentReportId, assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the URL of an assessment report in Audit Manager.",
            ),
        ),
        (
            "GetChangeLogs",
            deferred(
                "Model requires assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a list of changelogs from Audit Manager.",
            ),
        ),
        (
            "GetControl",
            deferred(
                "Model requires controlId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a specified control.",
            ),
        ),
        (
            "GetDelegations",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is delegations; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets a list of delegations from an audit owner to a delegate.",
            ),
        ),
        (
            "GetEvidence",
            deferred(
                "Model requires assessmentId, controlSetId, evidenceFolderId, evidenceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a specified evidence item.",
            ),
        ),
        (
            "GetEvidenceByEvidenceFolder",
            deferred(
                "Model requires assessmentId, controlSetId, evidenceFolderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets all evidence from a specified evidence folder in Audit Manager.",
            ),
        ),
        (
            "GetEvidenceFileUploadUrl",
            deferred(
                "Model requires fileName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a presigned Amazon S3 URL that can be used to upload a file as manual evidence. For instructions on how to use this operation, see Upload a file from your browser in the Audit Manager User Guide . The following…",
            ),
        ),
        (
            "GetEvidenceFolder",
            deferred(
                "Model requires assessmentId, controlSetId, evidenceFolderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an evidence folder from a specified assessment in Audit Manager.",
            ),
        ),
        (
            "GetEvidenceFoldersByAssessment",
            deferred(
                "Model requires assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the evidence folders from a specified assessment in Audit Manager.",
            ),
        ),
        (
            "GetEvidenceFoldersByAssessmentControl",
            deferred(
                "Model requires assessmentId, controlSetId, controlId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a list of evidence folders that are associated with a specified control in an Audit Manager assessment.",
            ),
        ),
        (
            "GetInsights",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is insights; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets the latest analytics data for all your current active assessments.",
            ),
        ),
        (
            "GetInsightsByAssessment",
            deferred(
                "Model requires assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the latest analytics data for a specific active assessment.",
            ),
        ),
        (
            "GetOrganizationAdminAccount",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is adminAccountId, organizationId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets the name of the delegated Amazon Web Services administrator account for a specified organization.",
            ),
        ),
        (
            "GetServicesInScope",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is serviceMetadata; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets a list of the Amazon Web Services services from which Audit Manager can collect evidence. Audit Manager defines which Amazon Web Services services are in scope for an assessment. Audit Manager infers this scope by examining…",
            ),
        ),
        (
            "GetSettings",
            deferred(
                "Model requires attribute. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the settings for a specified Amazon Web Services account.",
            ),
        ),
        (
            "ListAssessmentControlInsightsByControlDomain",
            deferred(
                "Model requires controlDomainId, assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the latest analytics data for controls within a specific control domain and a specific active assessment. Control insights are listed only if the control belongs to the control domain and assessment that was specified.…",
            ),
        ),
        (
            "ListAssessmentFrameworkShareRequests",
            deferred(
                "Model requires requestType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of sent or received share requests for custom frameworks in Audit Manager.",
            ),
        ),
        (
            "ListAssessmentFrameworks",
            OperationOverlay {
                fixed: [(
                    path("frameworkType")?,
                    FixedValue::String("Custom".to_owned()),
                )]
                .into_iter()
                .collect(),
                ..read(
                    "frameworkMetadataList",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListAssessmentReports",
            OperationOverlay {
                queryable: vec![path("id")?],
                ..read(
                    "assessmentReports",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListAssessments",
            OperationOverlay {
                queryable: vec![path("id")?],
                ..read(
                    "assessmentMetadata",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListControlDomainInsights",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is controlDomainInsights; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the latest analytics data for control domains across all of your active assessments. Audit Manager supports the control domains that are provided by Amazon Web Services Control Catalog. For information about how to find a…",
            ),
        ),
        (
            "ListControlDomainInsightsByAssessment",
            deferred(
                "Model requires assessmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists analytics data for control domains within a specified active assessment. Audit Manager supports the control domains that are provided by Amazon Web Services Control Catalog. For information about how to find a list of…",
            ),
        ),
        (
            "ListControlInsightsByControlDomain",
            deferred(
                "Model requires controlDomainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the latest analytics data for controls within a specific control domain across all active assessments. Control insights are listed only if the control belongs to the control domain that was specified and the control…",
            ),
        ),
        (
            "ListControls",
            OperationOverlay {
                fixed: [(
                    path("controlType")?,
                    FixedValue::String("Custom".to_owned()),
                )]
                .into_iter()
                .collect(),
                ..read(
                    "controlMetadataList",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListKeywordsForDataSource",
            deferred(
                "Model requires source. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of keywords that are pre-mapped to the specified control data source.",
            ),
        ),
        (
            "ListNotifications",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is notifications; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of all Audit Manager notifications.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags for the specified resource in Audit Manager.",
            ),
        ),
        (
            "RegisterAccount",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is status; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Enables Audit Manager for the specified Amazon Web Services account.",
            ),
        ),
        (
            "RegisterOrganizationAdminAccount",
            deferred(
                "Model requires adminAccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables an Amazon Web Services account within the organization as the delegated administrator for Audit Manager.",
            ),
        ),
        (
            "StartAssessmentFrameworkShare",
            deferred(
                "Model requires frameworkId, destinationAccount, destinationRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a share request for a custom framework in Audit Manager. The share request specifies a recipient and notifies them that a custom framework is available. Recipients have 120 days to accept or decline the request. If no…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tags the specified resource in Audit Manager.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a tag from a resource in Audit Manager.",
            ),
        ),
        (
            "UpdateAssessment",
            deferred(
                "Model requires assessmentId, scope. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Edits an Audit Manager assessment.",
            ),
        ),
        (
            "UpdateAssessmentControl",
            deferred(
                "Model requires assessmentId, controlSetId, controlId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a control within an assessment in Audit Manager.",
            ),
        ),
        (
            "UpdateAssessmentControlSetStatus",
            deferred(
                "Model requires assessmentId, controlSetId, status, comment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of a control set in an Audit Manager assessment.",
            ),
        ),
        (
            "UpdateAssessmentFramework",
            deferred(
                "Model requires frameworkId, name, controlSets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom framework in Audit Manager.",
            ),
        ),
        (
            "UpdateAssessmentFrameworkShare",
            deferred(
                "Model requires requestId, requestType, action. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a share request for a custom framework in Audit Manager.",
            ),
        ),
        (
            "UpdateAssessmentStatus",
            deferred(
                "Model requires assessmentId, status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of an assessment in Audit Manager.",
            ),
        ),
        (
            "UpdateControl",
            deferred(
                "Model requires controlId, name, controlMappingSources. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom control in Audit Manager.",
            ),
        ),
        (
            "UpdateSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is settings; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Updates Audit Manager settings for the current account.",
            ),
        ),
        (
            "ValidateAssessmentReportIntegrity",
            deferred(
                "Model requires s3RelativePath. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Validates the integrity of an assessment report in Audit Manager.",
            ),
        ),
    ])
}
