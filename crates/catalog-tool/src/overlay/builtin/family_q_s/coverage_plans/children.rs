use super::*;

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    match service.service.as_str() {
        "qbusiness" => qbusiness(service),
        "qconnect" => prompts(service),
        "s3files" => {
            let mut plan = arn("accessPoints", "accessPointArn", Some(("nextToken", "nextToken")))?;
            parent(&mut plan, "fileSystemId", "ListFileSystems", "fileSystemArn")?;
            plan.queryable.push(path("fileSystemId")?);
            put(service, "ListAccessPoints", plan)
        }
        "sagemaker" => {
            let mut plan = arn("ImageVersions", "ImageVersionArn", Some(("NextToken", "NextToken")))?;
            parent(&mut plan, "ImageName", "ListImages", "ImageName")?;
            plan.queryable.push(path("ImageArn")?);
            put(service, "ListImageVersions", plan)
        }
        "sesv2" => certificates(service),
        "service-catalog" => { principals(service)?; constraints(service) }
        _ => Ok(()),
    }
}

// Pin: qbusiness/service/2023-11-27/qbusiness-2023-11-27.json.
// Index/plugin/web summaries and data-source summaries contain no indexed documents or secrets.
fn qbusiness(service: &mut ServiceOverlay) -> Result<(), NameError> {
    [
        ("ListIndices", "indices", "indexId"),
        ("ListPlugins", "plugins", "pluginId"),
        ("ListWebExperiences", "webExperiences", "webExperienceId"),
        ("ListRetrievers", "retrievers", "retrieverId"),
    ].into_iter().try_for_each(|(operation, items, id)| {
        let mut plan = qualified(items, &["applicationId"], &[id], Some(("nextToken", "nextToken")))?;
        parent(&mut plan, "applicationId", "ListApplications", "applicationId")?;
        put(service, operation, plan)
    })?;
    let mut plan = qualified("dataSources", &["applicationId", "indexId"], &["dataSourceId"], Some(("nextToken", "nextToken")))?;
    parent(&mut plan, "indexId", "ListIndices", "indexId")?;
    inherited(&mut plan, "applicationId", "ListIndices", "applicationId")?;
    put(service, "ListDataSources", plan)
}

// Pin: qconnect/service/2020-10-19/qconnect-2020-10-19.json.
// AIPromptSummary has model/template TYPE metadata, not the template text in GetAIPrompt.
fn prompts(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut plan = arn("aiPromptSummaries", "aiPromptArn", Some(("nextToken", "nextToken")))?;
    parent(&mut plan, "assistantId", "ListAssistants", "assistantId")?;
    plan.fixed.insert(path("origin")?, FixedValue::String("CUSTOMER".to_owned()));
    plan.queryable.extend([path("assistantId")?, path("aiPromptId")?]);
    plan.tags = Some(path("tags")?);
    put(service, "ListAIPrompts", plan)?;
    let identity = Identity::CorrelatedKey(vec![
        IdentityField::SelectedArn(path("aiPromptSummary.aiPromptArn")?),
        IdentityField::SelectedInteger { path: path("versionNumber")?, minimum: 1, maximum: i64::MAX },
    ]);
    let mut versions = list("aiPromptVersionSummaries", identity, Some(("nextToken", "nextToken")))?;
    parent(&mut versions, "aiPromptId", "ListAIPrompts", "aiPromptId")?;
    inherited(&mut versions, "assistantId", "ListAIPrompts", "assistantId")?;
    versions.queryable.push(path("aiPromptSummary.aiPromptArn")?);
    versions.fixed.insert(path("origin")?, FixedValue::String("CUSTOMER".to_owned()));
    put(service, "ListAIPromptVersions", versions)
}

// Pin: sesv2/service/2019-09-27/sesv2-2019-09-27.json.
// FromAddress distinguishes registrations under domain identities; email-only registrations may omit it.
// List returns certificate ARN, from-address, state and expiry only; no certificate/key bytes.
fn certificates(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut plan = qualified("Certificates", &["EmailIdentity"], &["CertificateArn"], Some(("NextToken", "NextToken")))?;
    plan.identity = Some(crate::overlay::Identity::Choice(crate::overlay::IdentityChoice::Discriminated(vec![
        crate::overlay::IdentityBranch::new(
            crate::overlay::IdentityPredicate::Present(path("FromAddress")?),
            crate::overlay::IdentityLeaf::Fields(vec![IdentityField::Request(path("EmailIdentity")?),
                IdentityField::SelectedArn(path("CertificateArn")?), IdentityField::Selected(path("FromAddress")?)]),
        )?,
        crate::overlay::IdentityBranch::new(
            crate::overlay::IdentityPredicate::Present(path("CertificateArn")?),
            crate::overlay::IdentityLeaf::Fields(vec![IdentityField::Request(path("EmailIdentity")?),
                IdentityField::SelectedArn(path("CertificateArn")?)]),
        )?,
    ])));
    parent(&mut plan, "EmailIdentity", "ListEmailIdentities", "IdentityName")?;
    put(service, "ListEmailIdentityCertificates", plan)
}

// Pin: service-catalog/service/2015-12-10/service-catalog-2015-12-10.json.
// IAM_PATTERN is an association selector, not necessarily an ARN of a concrete IAM principal.
fn principals(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut plan = qualified("Principals", &["PortfolioId"], &["PrincipalARN", "PrincipalType"], Some(("PageToken", "NextPageToken")))?;
    parent(&mut plan, "PortfolioId", "ListPortfolios", "Id")?;
    put(service, "ListPrincipalsForPortfolio", plan)
}

fn constraints(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut plan = qualified("ConstraintDetails", &["PortfolioId"], &["ConstraintId"], Some(("PageToken", "NextPageToken")))?;
    parent(&mut plan, "PortfolioId", "ListPortfolios", "Id")?;
    plan.classification = Some(crate::overlay::KindClassification {
        field: path("Type")?,
        variants: [
            ("LAUNCH".to_owned(), "AWS::ServiceCatalog::LaunchRoleConstraint".to_owned()),
            ("NOTIFICATION".to_owned(), "AWS::ServiceCatalog::LaunchNotificationConstraint".to_owned()),
            ("RESOURCE_UPDATE".to_owned(), "AWS::ServiceCatalog::ResourceUpdateConstraint".to_owned()),
            ("STACKSET".to_owned(), "AWS::ServiceCatalog::StackSetConstraint".to_owned()),
            ("TEMPLATE".to_owned(), "AWS::ServiceCatalog::LaunchTemplateConstraint".to_owned()),
        ].into(),
    });
    put(service, "ListConstraintsForPortfolio", plan)
}
