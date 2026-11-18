use super::{bind, inventory, path, paths, token};
use crate::overlay::{Identity, NameError, OperationOverlay, Paginator, Scope};

pub(super) fn get_assistant() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "assistant",
        Identity::Arn(path("assistantArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "assistantId",
        "assistantArn",
        "name",
        "type",
        "status",
        "description",
    ])?;
    operation.tags = Some(path("tags")?);
    bind(
        &mut operation,
        "assistantId",
        "ListAssistants",
        "assistantId",
    )?;
    Ok(operation)
}

pub(super) fn get_assistant_association() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "assistantAssociation",
        Identity::Arn(path("assistantAssociationArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "assistantAssociationId",
        "assistantAssociationArn",
        "assistantId",
        "assistantArn",
        "associationType",
        "associationData.knowledgeBaseAssociation.knowledgeBaseArn",
    ])?;
    operation.tags = Some(path("tags")?);
    bind(
        &mut operation,
        "assistantAssociationId",
        "ListAssistantAssociations",
        "assistantAssociationId",
    )?;
    bind(
        &mut operation,
        "assistantId",
        "ListAssistantAssociations",
        "assistantId",
    )?;
    Ok(operation)
}

pub(super) fn get_knowledge_base() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "knowledgeBase",
        Identity::Arn(path("knowledgeBaseArn")?),
        Paginator::None,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "knowledgeBaseId",
        "knowledgeBaseArn",
        "name",
        "knowledgeBaseType",
        "status",
        "description",
    ])?;
    operation.tags = Some(path("tags")?);
    bind(
        &mut operation,
        "knowledgeBaseId",
        "ListKnowledgeBases",
        "knowledgeBaseId",
    )?;
    Ok(operation)
}

pub(super) fn list_assistant_associations() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "assistantAssociationSummaries",
        Identity::Arn(path("assistantAssociationArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "assistantAssociationId",
        "assistantAssociationArn",
        "assistantId",
        "assistantArn",
        "associationType",
        "associationData.knowledgeBaseAssociation.knowledgeBaseArn",
    ])?;
    operation.tags = Some(path("tags")?);
    operation.page_size = Some(100);
    bind(
        &mut operation,
        "assistantId",
        "ListAssistants",
        "assistantId",
    )?;
    Ok(operation)
}

pub(super) fn list_assistants() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "assistantSummaries",
        Identity::Arn(path("assistantArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "assistantId",
        "assistantArn",
        "name",
        "type",
        "status",
        "description",
    ])?;
    operation.tags = Some(path("tags")?);
    operation.page_size = Some(100);
    Ok(operation)
}

pub(super) fn list_knowledge_bases() -> Result<OperationOverlay, NameError> {
    let mut operation = inventory(
        "knowledgeBaseSummaries",
        Identity::Arn(path("knowledgeBaseArn")?),
        token("nextToken", "nextToken")?,
        Scope::Region,
    )?;
    operation.queryable = paths(&[
        "knowledgeBaseId",
        "knowledgeBaseArn",
        "name",
        "knowledgeBaseType",
        "status",
        "description",
    ])?;
    operation.tags = Some(path("tags")?);
    operation.page_size = Some(100);
    Ok(operation)
}
