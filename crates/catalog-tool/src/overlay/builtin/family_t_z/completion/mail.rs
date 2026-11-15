use super::*;

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    // Pinned availability output is RedactedEwsAvailabilityProvider, never EwsPassword.
    // PersonalAccessTokenSummary contains only IDs, audit times and scopes, not tokens.
    aliases_and_permissions(service)?;
    [
        ("ListGroups", "Groups", &["Id"][..]),
        ("ListUsers", "Users", &["Id"][..]),
        ("ListResources", "Resources", &["Id"][..]),
        ("ListImpersonationRoles", "Roles", &["ImpersonationRoleId"][..]),
        ("ListMailDomains", "MailDomains", &["DomainName"][..]),
        ("ListAvailabilityConfigurations", "AvailabilityConfigurations", &["DomainName"][..]),
        ("ListMobileDeviceAccessOverrides", "Overrides", &["UserId", "DeviceId"][..]),
        ("ListPersonalAccessTokens", "PersonalAccessTokenSummaries", &["PersonalAccessTokenId"][..]),
    ].into_iter().try_for_each(|(operation, items, fields)| {
        let plan = child(items, correlated(&["OrganizationId"], fields)?, "ListOrganizations", "OrganizationId", "OrganizationId", true)?;
        insert(service, operation, plan)
    })?;
    [
        ("ListAccessControlRules", "Name"),
        ("ListMobileDeviceAccessRules", "MobileDeviceAccessRuleId"),
    ].into_iter().try_for_each(|(operation, id)| {
        let plan = child("Rules", correlated(&["OrganizationId"], &[id])?, "ListOrganizations", "OrganizationId", "OrganizationId", false)?;
        insert(service, operation, plan)
    })?;
    [
        ("ListGroupMembers", "Members", "ListGroups", "GroupId", "Id"),
        ("ListResourceDelegates", "Delegates", "ListResources", "ResourceId", "Id"),
    ].into_iter().try_for_each(|(operation, items, parent, input, id)| {
        let mut plan = child(items, correlated(&["OrganizationId", input], &[id])?, parent, input, "Id", true)?;
        request_context(&mut plan, "OrganizationId", parent, "OrganizationId")?;
        insert(service, operation, plan)
    })
}

fn aliases_and_permissions(service: &mut ServiceOverlay) -> Result<(), NameError> {
    [
        ("ListAliases", "Aliases", &["$"][..], &["ListUsers", "ListGroups", "ListResources"][..]),
        ("ListGroupsForEntity", "Groups", &["GroupId"][..], &["ListUsers", "ListGroups"][..]),
        ("ListMailboxPermissions", "Permissions", &["GranteeId"][..], &["ListUsers", "ListResources"][..]),
    ].into_iter().try_for_each(|(operation, items, ids, parents)| {
        let mut plan = inventory(items, correlated(&["OrganizationId", "EntityId"], ids)?, token("NextToken", "NextToken")?, Scope::Region)?;
        plan.class = Some(crate::overlay::OverlayClass::Expand);
        plan.bind.insert(path("EntityId")?, alternatives(parents, ContextSource::SelectedItem, "Id")?);
        plan.bind.insert(path("OrganizationId")?, alternatives(parents, ContextSource::Request, "OrganizationId")?);
        insert(service, operation, plan)
    })
}

fn alternatives(parents: &[&str], source: ContextSource, field: &str) -> Result<Binding, NameError> {
    Ok(Binding::FromAny(parents.iter().map(|operation| {
        Ok(crate::overlay::BindingAlternative { operation: name(operation)?, source, path: path(field)? })
    }).collect::<Result<Vec<_>, NameError>>()?))
}
