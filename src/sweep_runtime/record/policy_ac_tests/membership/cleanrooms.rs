mod associations;

use super::*;

const SPEC: [(&str, &str, &str, &str, &str); 7] = [
    (
        "ListAnalysisTemplates",
        "AWS::CleanRooms::AnalysisTemplate",
        "analysisTemplateSummaries",
        "analysistemplates",
        "analysistemplate",
    ),
    (
        "ListConfiguredAudienceModelAssociations",
        "AWS::ApiOnly::CleanRoomsConfiguredAudienceModelAssociation",
        "configuredAudienceModelAssociationSummaries",
        "configuredaudiencemodelassociations",
        "configuredaudiencemodelassociation",
    ),
    (
        "ListConfiguredTableAssociations",
        "AWS::CleanRooms::ConfiguredTableAssociation",
        "configuredTableAssociationSummaries",
        "configuredTableAssociations",
        "configuredtableassociation",
    ),
    (
        "ListIdMappingTables",
        "AWS::CleanRooms::IdMappingTable",
        "idMappingTableSummaries",
        "idmappingtables",
        "idmappingtable",
    ),
    (
        "ListIdNamespaceAssociations",
        "AWS::CleanRooms::IdNamespaceAssociation",
        "idNamespaceAssociationSummaries",
        "idnamespaceassociations",
        "idnamespaceassociation",
    ),
    (
        "ListIntermediateTables",
        "AWS::CleanRooms::IntermediateTable",
        "intermediateTableSummaries",
        "intermediateTables",
        "intermediatetable",
    ),
    (
        "ListPrivacyBudgetTemplates",
        "AWS::CleanRooms::PrivacyBudgetTemplate",
        "privacyBudgetTemplateSummaries",
        "privacybudgettemplates",
        "privacybudgettemplate",
    ),
];

const TABLES: [&str; 2] = [
    "44444444-4444-4444-8444-444444444444",
    "55555555-5555-4555-8555-555555555555",
];
const MODELS: [&str; 2] = [
    "arn:aws:cleanrooms-ml:us-east-1:123456789012:configured-audience-model/model-a",
    "arn:aws:cleanrooms-ml:us-east-1:123456789012:configured-audience-model/model-b",
];
const NAMESPACES: [&str; 2] = [
    "arn:aws:entityresolution:us-east-1:123456789012:idnamespace/namespace-a",
    "arn:aws:entityresolution:us-east-1:123456789012:idnamespace/namespace-b",
];

fn namespace_arn(owner: &str) -> TestResult<&'static str> {
    OWNERS
        .into_iter()
        .zip(NAMESPACES)
        .find_map(|(membership, arn)| (membership == owner).then_some(arn))
        .ok_or_else(|| "unreviewed namespace association owner".into())
}

fn endpoint_ids(owner: &str) -> TestResult<(&'static str, &'static str)> {
    OWNERS
        .into_iter()
        .zip(TABLES)
        .zip(MODELS)
        .find_map(|((membership, table), model)| (membership == owner).then_some((table, model)))
        .ok_or_else(|| "unreviewed association owner".into())
}

fn membership(owner: &str) -> Json {
    json!({"id":owner,"arn":format!("arn:aws:cleanrooms:us-east-1:123456789012:membership/{owner}"),
        "collaborationId":"33333333-3333-4333-8333-333333333333",
        "collaborationArn":"arn:aws:cleanrooms:us-east-1:123456789012:collaboration/33333333-3333-4333-8333-333333333333",
        "collaborationCreatorAccountId":"123456789012","collaborationCreatorDisplayName":"owner",
        "collaborationName":"metadata-only","createTime":1767225600,"updateTime":1767312000,
        "status":"ACTIVE","memberAbilities":["CAN_QUERY"],"paymentConfiguration":{"queryCompute":{"isResponsible":false}}})
}

fn summary(operation: &str, owner: &str, arn: &str, id: &str) -> TestResult<Json> {
    let (table, model) = endpoint_ids(owner)?;
    let mut row = json!({"arn":arn,"id":id,"name":"same","description":"configuration metadata",
        "membershipId":owner,"membershipArn":format!("arn:aws:cleanrooms:us-east-1:123456789012:membership/{owner}"),
        "collaborationId":"33333333-3333-4333-8333-333333333333",
        "collaborationArn":"arn:aws:cleanrooms:us-east-1:123456789012:collaboration/33333333-3333-4333-8333-333333333333",
        "createTime":1767225600,"updateTime":1767312000});
    let object = row.as_object_mut().ok_or("summary must be an object")?;
    match operation {
        "ListAnalysisTemplates" => {
            object.insert("isSyntheticData".to_owned(), json!(false));
        }
        "ListConfiguredAudienceModelAssociations" => {
            object.insert("configuredAudienceModelArn".to_owned(), json!(model));
        }
        "ListConfiguredTableAssociations" => {
            object.remove("description");
            object.remove("collaborationId");
            object.remove("collaborationArn");
            object.insert("configuredTableId".to_owned(), json!(table));
            object.insert("analysisRuleTypes".to_owned(), json!(["AGGREGATION"]));
        }
        "ListIdMappingTables" => {
            object.insert("inputReferenceConfig".to_owned(), json!({"inputReferenceArn":"arn:aws:entityresolution:us-east-1:123456789012:idmappingworkflow/workflow","manageResourcePolicies":false}));
        }
        "ListIdNamespaceAssociations" => {
            object.insert(
                "inputReferenceConfig".to_owned(),
                json!({"inputReferenceArn":namespace_arn(owner)?,"manageResourcePolicies":false}),
            );
            object.insert(
                "inputReferenceProperties".to_owned(),
                json!({"idNamespaceType":"SOURCE"}),
            );
        }
        "ListIntermediateTables" => {
            object.insert("status".to_owned(), json!("POPULATE_SUCCESS"));
            object.insert("retentionInDays".to_owned(), json!(7));
            object.insert("analysisRuleTypes".to_owned(), json!(["AGGREGATION"]));
        }
        "ListPrivacyBudgetTemplates" => {
            object.remove("description");
            object.remove("name");
            object.insert(
                "privacyBudgetType".to_owned(),
                json!("DIFFERENTIAL_PRIVACY"),
            );
        }
        _ => return Err("unreviewed Clean Rooms summary".into()),
    }
    Ok(row)
}

fn case(
    spec: (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
) -> TestResult<Case> {
    let (operation, kind, items, endpoint, resource) = spec;
    let owner = |owner: &str| -> TestResult<Owner> {
        let page = |later: bool| -> TestResult<Page> {
            let id = if later {
                "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
            } else {
                "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
            };
            let suffix = match operation {
                "ListPrivacyBudgetTemplates" => format!("privacybudgettemplate/{owner}-{id}"),
                "ListConfiguredTableAssociations" => {
                    format!("configuredtableassociation/{owner}/{id}")
                }
                _ => format!("membership/{owner}/{resource}/{id}"),
            };
            let arn = format!("arn:aws:cleanrooms:us-east-1:123456789012:{suffix}");
            let query = if later {
                vec!["nextToken=later"]
            } else {
                Vec::new()
            };
            Ok(Page {
                body: response(
                    items,
                    summary(operation, owner, &arn, id)?,
                    "nextToken",
                    later,
                ),
                id: arn,
                wire: NativeRequest::get(format!("/memberships/{owner}/{endpoint}"), &query),
            })
        };
        Ok(Owner {
            value: owner.to_owned(),
            pages: [page(false)?, page(true)?],
        })
    };
    Ok(Case {
        operation,
        kind,
        parent_field: "membershipIdentifier",
        requires_request_identity: false,
        owners: [owner(OWNERS[0])?, owner(OWNERS[1])?],
    })
}

#[tokio::test]
async fn given_two_memberships_when_all_seven_metadata_lists_resume_then_exact_owned_summary_arns_survive_sqlite()
-> TestResult {
    let cases = SPEC.into_iter().map(case).collect::<TestResult<Vec<_>>>()?;
    let excluded = [
        "GetAnalysisTemplate",
        "StartProtectedQuery",
        "PopulateIntermediateTable",
    ];
    let operations = std::iter::once("ListMemberships")
        .chain(SPEC.iter().map(|spec| spec.0))
        .chain(excluded)
        .collect::<Vec<_>>();
    run(
        "cleanrooms",
        "2022-02-17",
        "ListMemberships",
        "id",
        json!({"membershipSummaries":OWNERS.map(membership)}),
        &operations,
        &excluded,
        cases,
    )
    .await
}
