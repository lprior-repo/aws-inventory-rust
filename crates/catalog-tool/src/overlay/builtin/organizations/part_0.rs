use super::super::{empty, name, path};
use crate::overlay::{
    Binding, FixedValue, Identity, IdentityField, ItemSelector, NameError, OperationOverlay,
    OverlayClass, Paginator, Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeAccount", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Account")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        bind: [(path("AccountId")?, Binding::From { operation: name("ListAccounts")?, path: path("Id")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        exclude_response_members: vec![path("Account.Email")?],
        ..empty()
    }),
("DescribeCreateAccountStatus", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("CreateAccountStatus")?),
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("CreateAccountRequestId")?, Binding::From { operation: name("ListCreateAccountStatus")?, path: path("Id")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeOrganization", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Organization")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        exclude_response_members: vec![path("Organization.MasterAccountEmail")?],
        ..empty()
    }),
("DescribeResourcePolicy", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ResourcePolicy")?),
        identity: Some(Identity::Arn(path("ResourcePolicySummary.Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListAWSServiceAccessForOrganization", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("EnabledServicePrincipals")?),
        page_size: Some(20),
        identity: Some(Identity::Key(vec![path("ServicePrincipal")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListAccounts", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Accounts")?),
        page_size: Some(20),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        exclude_response_members: vec![path("Accounts.Email")?],
        ..empty()
    }),
("ListCreateAccountStatus", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("CreateAccountStatuses")?),
        page_size: Some(20),
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListDelegatedAdministrators", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("DelegatedAdministrators")?),
        page_size: Some(20),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        exclude_response_members: vec![path("DelegatedAdministrators.Email")?],
        ..empty()
    }),
("ListInboundResponsibilityTransfers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ResponsibilityTransfers")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        fixed: [(path("Type")?, FixedValue::String("BILLING".to_owned())), (path("MaxResults")?, FixedValue::Integer(20))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        exclude_response_members: vec![path("ResponsibilityTransfers.Source.ManagementAccountEmail")?, path("ResponsibilityTransfers.Target.ManagementAccountEmail")?],
        ..empty()
    }),
("ListOutboundResponsibilityTransfers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ResponsibilityTransfers")?),
        identity: Some(Identity::Arn(path("Arn")?)),
        fixed: [(path("Type")?, FixedValue::String("BILLING".to_owned())), (path("MaxResults")?, FixedValue::Integer(20))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        exclude_response_members: vec![path("ResponsibilityTransfers.Source.ManagementAccountEmail")?, path("ResponsibilityTransfers.Target.ManagementAccountEmail")?],
        ..empty()
    }),
("ListRoots", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Roots")?),
        page_size: Some(20),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeEffectivePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("EffectivePolicy contains management configuration, not workload content. Required scalar PolicyType needs fan-out over all 13 modeled EffectivePolicyType values; optional TargetId should bind ListAccounts.Id for management-account collection. Current fixed arrays cannot fan out a scalar. EffectivePolicy.TargetId plus PolicyType supplies identity once supported. SCP and RCP are not supported by this API.".to_owned()),
        ..empty()
    }),
("DescribeOrganizationalUnit", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required OrganizationalUnitId must bind ListOrganizationalUnitsForParent.OrganizationalUnits.Id; exhaustive OU discovery needs bounded recursive parent traversal, which the binding schema rejects. Output OrganizationalUnit.Arn supplies identity once the complete source exists.".to_owned()),
        ..empty()
    }),
("DescribePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Policy.Content is organization policy configuration, not workload content. Required PolicyId needs the complete ListPolicies.Policies.Id source, currently blocked by scalar policy-type fan-out. Output Policy.PolicySummary.Arn supplies identity. No fixed policy ID or one-policy-type shortcut is authorized.".to_owned()),
        ..empty()
    }),
("DescribeResponsibilityTransfer", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required Id needs the union of ListInboundResponsibilityTransfers.Id and ListOutboundResponsibilityTransfers.Id. Current binding accepts one source only; selecting either loses transfers in the other direction. Output ResponsibilityTransfer.Arn supplies identity.".to_owned()),
        ..empty()
    }),
("ListAccountsForParent", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Lists immediate accounts only. Required ParentId needs the union of ListRoots.Id and every recursively discovered organizational unit Id; roots-only binding omits nested accounts. ParentId is absent from returned Accounts items, so membership edges also require preserved request-parent context. ListAccounts remains the exhaustive account inventory source.".to_owned()),
        ..empty()
    }),
("ListAccountsWithInvalidEffectivePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required scalar PolicyType needs fan-out over all 13 modeled EffectivePolicyType values. Returned Accounts have Arn identities, but the response-level PolicyType must be retained to represent which policy type is invalid. Fixed arrays are not scalar fan-out and a single type is incomplete.".to_owned()),
        ..empty()
    }),
("ListChildren", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required ParentId needs roots plus all recursively discovered OUs; required ChildType needs both ACCOUNT and ORGANIZATIONAL_UNIT scalar invocations. Children contains Id and Type but no parent ID. Complete traversal and parent-child edges need bounded recursion, source unions, enum fan-out and request-parent context, none expressible by current binding and identity tables.".to_owned()),
        ..empty()
    }),
("ListDelegatedServicesForAccount", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("DelegatedServices")?),
        identity: Some(Identity::CorrelatedKey(vec![
            IdentityField::Request(path("AccountId")?),
            IdentityField::Selected(path("ServicePrincipal")?),
        ])),
        bind: [(path("AccountId")?, Binding::From {
            operation: name("ListDelegatedAdministrators")?, path: path("Id")?,
        })].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair {
            input: path("NextToken")?, output: path("NextToken")?, optional: false,
        })),
        ..empty()
    }),
("ListEffectivePolicyValidationErrors", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required AccountId can bind ListAccounts.Id but scalar PolicyType needs all 13 EffectivePolicyType values. Error items omit both request identity fields, while response-level AccountId, PolicyType, Path and EvaluationTimestamp must be retained. Need request/response context in diagnostic identity and contributed-policy relations; exclude EvaluationTimestamp from change hashing once representable.".to_owned()),
        ..empty()
    })
    ])
}
