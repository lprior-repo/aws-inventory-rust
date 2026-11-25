use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchGetCollaborationAnalysisTemplate",
            deferred(
                "Model requires collaborationIdentifier, analysisTemplateArns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves multiple analysis templates within a collaboration by their Amazon Resource Names (ARNs).",
            ),
        ),
        (
            "BatchGetSchema",
            deferred(
                "Model requires collaborationIdentifier, names. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves multiple schemas by their identifiers.",
            ),
        ),
        (
            "BatchGetSchemaAnalysisRule",
            deferred(
                "Model requires collaborationIdentifier, schemaAnalysisRuleRequests. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves multiple analysis rule schemas.",
            ),
        ),
        (
            "CreateAnalysisTemplate",
            deferred(
                "Model requires membershipIdentifier, name, format, source. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new analysis template.",
            ),
        ),
        (
            "CreateCollaboration",
            deferred(
                "Model requires members, name, creatorMemberAbilities, creatorDisplayName, queryLogStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new collaboration.",
            ),
        ),
        (
            "CreateCollaborationChangeRequest",
            deferred(
                "Model requires collaborationIdentifier, changes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new change request to modify an existing collaboration. This enables post-creation modifications to collaborations through a structured API-driven approach.",
            ),
        ),
        (
            "CreateConfiguredAudienceModelAssociation",
            deferred(
                "Model requires membershipIdentifier, configuredAudienceModelArn, configuredAudienceModelAssociationName, manageResourcePolicies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the details necessary to create a configured audience model association.",
            ),
        ),
        (
            "CreateConfiguredTable",
            deferred(
                "Model requires name, tableReference, allowedColumns, analysisMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new configured table resource.",
            ),
        ),
        (
            "CreateConfiguredTableAnalysisRule",
            deferred(
                "Model requires configuredTableIdentifier, analysisRuleType, analysisRulePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new analysis rule for a configured table. Currently, only one analysis rule can be created for a given configured table.",
            ),
        ),
        (
            "CreateConfiguredTableAssociation",
            deferred(
                "Model requires name, membershipIdentifier, configuredTableIdentifier, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a configured table association. A configured table association links a configured table with a collaboration.",
            ),
        ),
        (
            "CreateConfiguredTableAssociationAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, configuredTableAssociationIdentifier, analysisRuleType, analysisRulePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new analysis rule for an associated configured table.",
            ),
        ),
        (
            "CreateIdMappingTable",
            deferred(
                "Model requires membershipIdentifier, name, inputReferenceConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an ID mapping table.",
            ),
        ),
        (
            "CreateIdNamespaceAssociation",
            deferred(
                "Model requires membershipIdentifier, inputReferenceConfig, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an ID namespace association.",
            ),
        ),
        (
            "CreateIntermediateTable",
            deferred(
                "Model requires membershipIdentifier, name, populationAnalysisConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an intermediate table in a membership. The intermediate table is owned by the member with the CAN_QUERY ability. To populate the table with results, use PopulateIntermediateTable .",
            ),
        ),
        (
            "CreateIntermediateTableAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, intermediateTableIdentifier, analysisRuleType, analysisRulePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an analysis rule for an intermediate table. Only the CUSTOM analysis rule type is supported. Only the intermediate table owner can call this operation.",
            ),
        ),
        (
            "CreateMembership",
            deferred(
                "Model requires collaborationIdentifier, queryLogStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a membership for a specific collaboration identifier and joins the collaboration.",
            ),
        ),
        (
            "CreatePrivacyBudgetTemplate",
            deferred(
                "Model requires membershipIdentifier, privacyBudgetType, parameters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a privacy budget template for a specified collaboration. Each collaboration can have only one privacy budget template. If you need to change the privacy budget template, use the UpdatePrivacyBudgetTemplate operation.",
            ),
        ),
        (
            "DeleteAnalysisTemplate",
            deferred(
                "Model requires membershipIdentifier, analysisTemplateIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an analysis template.",
            ),
        ),
        (
            "DeleteCollaboration",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a collaboration. It can only be called by the collaboration owner.",
            ),
        ),
        (
            "DeleteConfiguredAudienceModelAssociation",
            deferred(
                "Model requires configuredAudienceModelAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the information necessary to delete a configured audience model association.",
            ),
        ),
        (
            "DeleteConfiguredTable",
            deferred(
                "Model requires configuredTableIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configured table.",
            ),
        ),
        (
            "DeleteConfiguredTableAnalysisRule",
            deferred(
                "Model requires configuredTableIdentifier, analysisRuleType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configured table analysis rule.",
            ),
        ),
        (
            "DeleteConfiguredTableAssociation",
            deferred(
                "Model requires configuredTableAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configured table association.",
            ),
        ),
        (
            "DeleteConfiguredTableAssociationAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, configuredTableAssociationIdentifier, analysisRuleType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an analysis rule for a configured table association.",
            ),
        ),
        (
            "DeleteIdMappingTable",
            deferred(
                "Model requires idMappingTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an ID mapping table.",
            ),
        ),
        (
            "DeleteIdNamespaceAssociation",
            deferred(
                "Model requires idNamespaceAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an ID namespace association.",
            ),
        ),
        (
            "DeleteIntermediateTable",
            deferred(
                "Model requires membershipIdentifier, intermediateTableIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an intermediate table. The delete is idempotent. Only the intermediate table owner can call this operation.",
            ),
        ),
        (
            "DeleteIntermediateTableAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, intermediateTableIdentifier, analysisRuleType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an analysis rule from an intermediate table. After the analysis rule is deleted, the intermediate table becomes unqueryable until a new analysis rule is attached. Only the intermediate table owner can call this operation.",
            ),
        ),
        (
            "DeleteMember",
            deferred(
                "Model requires collaborationIdentifier, accountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified member from a collaboration. The removed member is placed in the Removed status and can't interact with the collaboration. The removed member's data is inaccessible to active members of the collaboration.",
            ),
        ),
        (
            "DeleteMembership",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specified membership. All resources under a membership must be deleted.",
            ),
        ),
        (
            "DeletePrivacyBudgetTemplate",
            deferred(
                "Model requires membershipIdentifier, privacyBudgetTemplateIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a privacy budget template for a specified collaboration.",
            ),
        ),
        (
            "DisallowIntermediateTable",
            deferred(
                "Model requires membershipIdentifier, intermediateTableName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Marks an intermediate table as invalid when it references the caller's base table. The data provider (base table owner) calls this operation, not the intermediate table owner. By default, the operation also marks all descendant…",
            ),
        ),
        (
            "GetAnalysisLogExport",
            deferred(
                "Model requires membershipIdentifier, analysisLogExportIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an analysis log export, including its current status and, if the export failed, the reason for the failure. Poll this operation until the status is SUCCESS or FAILED . An export can't be canceled after…",
            ),
        ),
        (
            "GetAnalysisTemplate",
            deferred(
                "Model requires membershipIdentifier, analysisTemplateIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an analysis template.",
            ),
        ),
        (
            "GetCollaboration",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("collaborationIdentifier")?,
                    Binding::From {
                        operation: name("ListCollaborations")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "collaboration",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetCollaborationAnalysisTemplate",
            deferred(
                "Model requires collaborationIdentifier, analysisTemplateArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an analysis template within a collaboration.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetCollaborationChangeRequest",
            deferred(
                "Model requires collaborationIdentifier, changeRequestIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific collaboration change request.",
            ),
        ),
        (
            "GetCollaborationConfiguredAudienceModelAssociation",
            deferred(
                "Model requires collaborationIdentifier, configuredAudienceModelAssociationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a configured audience model association within a collaboration.",
            ),
        ),
        (
            "GetCollaborationIdNamespaceAssociation",
            deferred(
                "Model requires collaborationIdentifier, idNamespaceAssociationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an ID namespace association from a specific collaboration.",
            ),
        ),
        (
            "GetCollaborationPrivacyBudgetTemplate",
            deferred(
                "Model requires collaborationIdentifier, privacyBudgetTemplateIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns details about a specified privacy budget template.",
            ),
        ),
        (
            "GetConfiguredAudienceModelAssociation",
            deferred(
                "Model requires configuredAudienceModelAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a configured audience model association.",
            ),
        ),
        (
            "GetConfiguredTable",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("configuredTableIdentifier")?,
                    Binding::From {
                        operation: name("ListConfiguredTables")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "configuredTable",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetConfiguredTableAnalysisRule",
            deferred(
                "Model requires configuredTableIdentifier, analysisRuleType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a configured table analysis rule.",
            ),
        ),
        (
            "GetConfiguredTableAssociation",
            deferred(
                "Model requires configuredTableAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a configured table association.",
            ),
        ),
        (
            "GetConfiguredTableAssociationAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, configuredTableAssociationIdentifier, analysisRuleType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the analysis rule for a configured table association.",
            ),
        ),
        (
            "GetIdMappingTable",
            deferred(
                "Model requires idMappingTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an ID mapping table.",
            ),
        ),
        (
            "GetIdNamespaceAssociation",
            deferred(
                "Model requires idNamespaceAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an ID namespace association.",
            ),
        ),
        (
            "GetIntermediateTable",
            deferred(
                "Model requires intermediateTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an intermediate table. Returns the full details of the intermediate table, including schema, table dependencies, inherited constraints, child resources, and status. Only the intermediate table owner can call this…",
            ),
        ),
        (
            "GetIntermediateTableAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, intermediateTableIdentifier, analysisRuleType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the analysis rule for an intermediate table.",
            ),
        ),
        (
            "GetMembership",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "membership",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetPrivacyBudgetTemplate",
            deferred(
                "Model requires membershipIdentifier, privacyBudgetTemplateIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns details for a specified privacy budget template.",
            ),
        ),
        (
            "GetProtectedJob",
            deferred(
                "Model requires membershipIdentifier, protectedJobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns job processing metadata.",
            ),
        ),
        (
            "GetProtectedQuery",
            deferred(
                "Model requires membershipIdentifier, protectedQueryIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns query processing metadata.",
            ),
        ),
        (
            "GetSchema",
            deferred(
                "Model requires collaborationIdentifier, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the schema for a relation within a collaboration.",
            ),
        ),
        (
            "GetSchemaAnalysisRule",
            deferred(
                "Model requires collaborationIdentifier, name, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a schema analysis rule.",
            ),
        ),
        (
            "ListAnalysisLogExports",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists analysis log exports, sorted by the most recent export. Results are paginated. Use the nextToken parameter to retrieve additional results.",
            ),
        ),
        (
            "ListAnalysisTemplates",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "analysisTemplateSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListCollaborationAnalysisTemplates",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists analysis templates within a collaboration.",
            ),
        ),
        (
            "ListCollaborationChangeRequests",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all change requests for a collaboration with pagination support. Returns change requests sorted by creation time.",
            ),
        ),
        (
            "ListCollaborationConfiguredAudienceModelAssociations",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists configured audience model associations within a collaboration.",
            ),
        ),
        (
            "ListCollaborationIdNamespaceAssociations",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the ID namespace associations in a collaboration.",
            ),
        ),
        (
            "ListCollaborationPrivacyBudgetTemplates",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an array that summarizes each privacy budget template in a specified collaboration.",
            ),
        ),
        (
            "ListCollaborationPrivacyBudgets",
            deferred(
                "Model requires collaborationIdentifier, privacyBudgetType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an array that summarizes each privacy budget in a specified collaboration. The summary includes the collaboration ARN, creation time, creating account, and privacy budget details.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListCollaborations",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "collaborationList",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListConfiguredAudienceModelAssociations",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?, path("configuredAudienceModelArn")?],
                ..read(
                    "configuredAudienceModelAssociationSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListConfiguredTableAssociations",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?, path("configuredTableId")?],
                ..read(
                    "configuredTableAssociationSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListConfiguredTables",
            OperationOverlay {
                queryable: vec![path("arn")?, path("id")?],
                ..read(
                    "configuredTableSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListIdMappingTables",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "idMappingTableSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListIdNamespaceAssociations",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![
                    path("arn")?,
                    path("inputReferenceConfig.inputReferenceArn")?,
                ],
                ..read(
                    "idNamespaceAssociationSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListIntermediateTableVersions",
            deferred(
                "Model requires membershipIdentifier, intermediateTableIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the version history of an intermediate table. Each call to PopulateIntermediateTable creates a new version. We recommend using pagination to ensure that the operation returns quickly and successfully.",
            ),
        ),
        (
            "ListIntermediateTables",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "intermediateTableSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListMembers",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all members within a collaboration.",
            ),
        ),
        (
            "ListMemberships",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "membershipSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPrivacyBudgetTemplates",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("membershipIdentifier")?,
                    Binding::From {
                        operation: name("ListMemberships")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "privacyBudgetTemplateSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPrivacyBudgets",
            deferred(
                "Model requires membershipIdentifier, privacyBudgetType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed information about the privacy budgets in a specified membership.",
            ),
        ),
        (
            "ListProtectedJobs",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists protected jobs, sorted by most recent job.",
            ),
        ),
        (
            "ListProtectedQueries",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists protected queries, sorted by the most recent query.",
            ),
        ),
        (
            "ListSchemas",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the schemas for relations within a collaboration.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the tags that have been added to a resource.",
            ),
        ),
        (
            "PopulateIdMappingTable",
            deferred(
                "Model requires idMappingTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines the information that's necessary to populate an ID mapping table.",
            ),
        ),
        (
            "PopulateIntermediateTable",
            deferred(
                "Model requires intermediateTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Runs the stored query of an intermediate table and makes the results available for querying. Each call creates a new version. Use GetProtectedQuery with the returned analysis ID to track progress. Only the intermediate table…",
            ),
        ),
        (
            "PreviewPrivacyImpact",
            deferred(
                "Model requires membershipIdentifier, parameters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An estimate of the number of aggregation functions that the member who can query can run given epsilon and noise parameters.",
            ),
        ),
        (
            "StartAnalysisLogExport",
            deferred(
                "Model requires membershipIdentifier, analysisId, analysisType, resultConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an export of the Apache Spark logs for a protected query to an Amazon S3 bucket that you own. Use the exported logs to diagnose a query that failed or that ran more slowly than you expected. Clean Rooms exports a redacted…",
            ),
        ),
        (
            "StartProtectedJob",
            deferred(
                "Model requires type, membershipIdentifier, jobParameters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a protected job that is started by Clean Rooms.",
            ),
        ),
        (
            "StartProtectedQuery",
            deferred(
                "Model requires type, membershipIdentifier, sqlParameters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a protected query that is started by Clean Rooms.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tags a resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a tag or list of tags from a resource.",
            ),
        ),
        (
            "UpdateAnalysisTemplate",
            deferred(
                "Model requires membershipIdentifier, analysisTemplateIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the analysis template metadata.",
            ),
        ),
        (
            "UpdateCollaboration",
            deferred(
                "Model requires collaborationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates collaboration metadata and can only be called by the collaboration owner.",
            ),
        ),
        (
            "UpdateCollaborationChangeRequest",
            deferred(
                "Model requires collaborationIdentifier, changeRequestIdentifier, action. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing collaboration change request. This operation allows approval actions for pending change requests in collaborations (APPROVE, DENY, CANCEL, COMMIT). For change requests without automatic approval, a member in…",
            ),
        ),
        (
            "UpdateConfiguredAudienceModelAssociation",
            deferred(
                "Model requires configuredAudienceModelAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the details necessary to update a configured audience model association.",
            ),
        ),
        (
            "UpdateConfiguredTable",
            deferred(
                "Model requires configuredTableIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a configured table.",
            ),
        ),
        (
            "UpdateConfiguredTableAnalysisRule",
            deferred(
                "Model requires configuredTableIdentifier, analysisRuleType, analysisRulePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a configured table analysis rule.",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateConfiguredTableAssociation",
            deferred(
                "Model requires configuredTableAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a configured table association.",
            ),
        ),
        (
            "UpdateConfiguredTableAssociationAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, configuredTableAssociationIdentifier, analysisRuleType, analysisRulePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the analysis rule for a configured table association.",
            ),
        ),
        (
            "UpdateIdMappingTable",
            deferred(
                "Model requires idMappingTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the details that are necessary to update an ID mapping table.",
            ),
        ),
        (
            "UpdateIdNamespaceAssociation",
            deferred(
                "Model requires idNamespaceAssociationIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the details that are necessary to update an ID namespace association.",
            ),
        ),
        (
            "UpdateIntermediateTable",
            deferred(
                "Model requires intermediateTableIdentifier, membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an intermediate table. You can update the description, KMS key ARN, and column types of existing columns. Only the intermediate table owner can call this operation.",
            ),
        ),
        (
            "UpdateIntermediateTableAnalysisRule",
            deferred(
                "Model requires membershipIdentifier, intermediateTableIdentifier, analysisRuleType, analysisRulePolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the analysis rule policy for an intermediate table. Only the intermediate table owner can call this operation.",
            ),
        ),
        (
            "UpdateMembership",
            deferred(
                "Model requires membershipIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a membership.",
            ),
        ),
        (
            "UpdatePrivacyBudgetTemplate",
            deferred(
                "Model requires membershipIdentifier, privacyBudgetTemplateIdentifier, privacyBudgetType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the privacy budget template for the specified collaboration.",
            ),
        ),
        (
            "UpdateProtectedJob",
            deferred(
                "Model requires membershipIdentifier, protectedJobIdentifier, targetStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the processing of a currently running job.",
            ),
        ),
        (
            "UpdateProtectedQuery",
            deferred(
                "Model requires membershipIdentifier, protectedQueryIdentifier, targetStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the processing of a currently running query.",
            ),
        ),
    ])
}
