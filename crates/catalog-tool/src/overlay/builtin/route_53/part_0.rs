use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, BindingTransform, ChildEligibility, ContextSource, Identity, IdentityField,
    ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, ResourceEmission,
    RowOperand, RowPolicy, RowPredicate, RowValueType, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListHostedZones", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("HostedZones")?),
        page_size: Some(5),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("Name")?, path("Config.PrivateZone")?, path("LinkedService.ServicePrincipal")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("GetHostedZone", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("HostedZone")?),
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("Id")?, Binding::Transformed {
            binding: Box::new(Binding::From { operation: name("ListHostedZones")?, path: path("Id")? }),
            transform: BindingTransform::StripPrefix("/hostedzone/".to_owned()),
        })].into_iter().collect(),
        quota_group: Some("route53".to_owned()),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListHealthChecks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("HealthChecks")?),
        page_size: Some(5),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HealthCheckConfig.Type")?, path("HealthCheckConfig.IPAddress")?, path("HealthCheckConfig.FullyQualifiedDomainName")?, path("HealthCheckConfig.ChildHealthChecks")?, path("HealthCheckConfig.AlarmIdentifier.Name")?, path("HealthCheckConfig.AlarmIdentifier.Region")?, path("HealthCheckConfig.RoutingControlArn")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false }], truncated: Some(path("IsTruncated")?) }),
        ..empty()
    }),
("GetHealthCheck", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("HealthCheck")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HealthCheckConfig.Type")?, path("HealthCheckConfig.ChildHealthChecks")?, path("HealthCheckConfig.AlarmIdentifier.Name")?, path("HealthCheckConfig.AlarmIdentifier.Region")?, path("HealthCheckConfig.RoutingControlArn")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("HealthCheckId")?, Binding::From { operation: name("ListHealthChecks")?, path: path("Id")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListCidrCollections", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("CidrCollections")?),
        page_size: Some(5),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Arn")?, path("Id")?, path("Name")?, path("Version")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListQueryLoggingConfigs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("QueryLoggingConfigs")?),
        page_size: Some(5),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HostedZoneId")?, path("CloudWatchLogsLogGroupArn")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("GetQueryLoggingConfig", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("QueryLoggingConfig")?),
        quota_group: Some("route53".to_owned()),
        queryable: vec![path("Id")?, path("HostedZoneId")?, path("CloudWatchLogsLogGroupArn")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("Id")?, Binding::From { operation: name("ListQueryLoggingConfigs")?, path: path("Id")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetAccountLimit", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Account quota metadata requires Type enumeration across five AccountLimitType values; no per-operation variant fanout or account-scoped aggregate identity is supported. A single fixed Type would omit other limits.".to_owned()),
        ..empty()
    }),
("GetChange", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ChangeInfo is read-only status but required Id comes from mutating change responses, not an inventory enumerator. No account change-list operation exists; inventory must not manufacture changes to obtain IDs.".to_owned()),
        ..empty()
    }),
("GetCheckerIpRanges", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetDNSSEC", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Status")?),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::Request(path("HostedZoneId")?)])),
        bind: [(path("HostedZoneId")?, Binding::Transformed {
            binding: Box::new(Binding::From { operation: name("ListHostedZones")?, path: path("Id")? }),
            transform: BindingTransform::StripPrefix("/hostedzone/".to_owned()),
        })].into(),
        queryable: vec![path("ServeSignature")?],
        row_policy: RowPolicy {
            child: Some(ChildEligibility {
                operation: name("ListHostedZones")?,
                predicate: RowPredicate::BooleanEquals {
                    operand: RowOperand {
                        source: ContextSource::SelectedItem,
                        path: path("Config.PrivateZone")?,
                        value_type: RowValueType::Boolean,
                    },
                    value: false,
                },
            }),
            ..RowPolicy::default()
        },
        paginator: Some(Paginator::None),
        additional_resources: vec![ResourceEmission::new("key_signing_keys",
            ItemSelector::parse("KeySigningKeys")?, Identity::CorrelatedKey(vec![
                IdentityField::Request(path("HostedZoneId")?),
                IdentityField::Selected(path("Name")?),
            ]))],
        ..empty()
    }),
("GetGeoLocation", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetHealthCheckCount", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("HealthCheckCount is an account aggregate, not health-check identity. Root output has no stable account member; runtime compiler rejects contextual identity templates. ListHealthChecks supplies actual resource inventory.".to_owned()),
        ..empty()
    }),
("GetHealthCheckLastFailureReason", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("HealthCheckObservations contain checker Region, IPAddress, and StatusReport but omit parent HealthCheckId. Need parent-scoped identity and eligibility filtering: documentation excludes calculated health checks. This is diagnostic observation data, not configuration discovery.".to_owned()),
        ..empty()
    }),
("GetHealthCheckStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("HealthCheckObservations omit HealthCheckId, so checker Region/IPAddress alone collide across checks. Need parent-input identity and diagnostic scheduling; documentation explicitly excludes high-query-rate production use.".to_owned()),
        ..empty()
    }),
("GetHostedZoneCount", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("HostedZoneCount is an account aggregate without stable account identity in output. Runtime compiler rejects contextual identity templates; ListHostedZones supplies actual resources.".to_owned()),
        ..empty()
    }),
("GetHostedZoneLimit", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required HostedZoneId can bind ListHostedZones.Id, but Type requires both HostedZoneLimitType variants. Limit/Count output omits zone identity; need correlated Type fanout and parent-input identity rather than inventing a resource key from Count.".to_owned()),
        ..empty()
    }),
("GetReusableDelegationSet", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("DelegationSet")?),
        identity: Some(Identity::Key(vec![path("Id")?])),
        bind: [(path("Id")?, Binding::Transformed {
            binding: Box::new(Binding::From { operation: name("ListReusableDelegationSets")?, path: path("Id")? }),
            transform: BindingTransform::StripPrefix("/delegationset/".to_owned()),
        })].into_iter().collect(),
        quota_group: Some("route53".to_owned()),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetReusableDelegationSetLimit", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required DelegationSetId can bind ListReusableDelegationSets.Id and Type is MAX_ZONES_BY_REUSABLE_DELEGATION_SET, but Limit/Count output omits delegation-set ID, requiring parent-input identity.".to_owned()),
        ..empty()
    }),
("GetTrafficPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TrafficPolicy.Document is opaque JSON-encoded routing configuration, not a credential, but cannot be safely inspected or projected by this overlay. Keep deferred with ListTrafficPolicyVersions until generic safe-document policy and graph extraction are reviewed; do not parse arbitrary runtime JSON or silently drop Document.".to_owned()),
        ..empty()
    })
    ])
}
