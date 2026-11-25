use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeAccountLimits", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Limits")?),
        fixed: [(path("PageSize")?, FixedValue::Integer(400))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("Name")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("DescribeLoadBalancers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LoadBalancers")?),
        fixed: [(path("PageSize")?, FixedValue::Integer(400))].into_iter().collect(),
        identity: Some(Identity::Arn(path("LoadBalancerArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("DescribeTargetGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TargetGroups")?),
        fixed: [(path("PageSize")?, FixedValue::Integer(400))].into_iter().collect(),
        identity: Some(Identity::Arn(path("TargetGroupArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("DescribeTrustStores", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TrustStores")?),
        page_size: Some(400),
        identity: Some(Identity::Arn(path("TrustStoreArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("DescribeCapacityReservation", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("DescribeCapacityReservationOutput has no LoadBalancerArn; binding DescribeLoadBalancers.LoadBalancerArn into the request is possible, but identity cannot reference that bound parent ARN for the singleton reservation configuration.".to_owned()),
        ..empty()
    }),
("DescribeListenerAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Attributes contains only Key and Value and no ListenerArn; requires parent-input identity or parent-resource enrichment and a safe listener discovery source because DescribeListeners reaches OIDC ClientSecret.".to_owned()),
        ..empty()
    }),
("DescribeListenerCertificates", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires HTTPS/TLS listener selection and a safe listener discovery source; Certificates lacks ListenerArn, and the same CertificateArn can occur twice with different IsDefault values or on multiple listeners, requiring parent-scoped association identity.".to_owned()),
        ..empty()
    }),
("DescribeListeners", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeLoadBalancerAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Attributes contains only Key and Value; identical keys occur on different load balancers. Requires request LoadBalancerArn in identity or enrichment of the parent resource rather than globally identifying an attribute by Key.".to_owned()),
        ..empty()
    }),
("DescribeRules", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeSSLPolicies", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ResourceArns must cover load balancers, target groups, listeners, and rules; one from/path binding cannot union these sources or enrich their tags. Safe listener/rule discovery is also blocked by reachable OIDC ClientSecret; enabling only one ARN source would silently omit the others.".to_owned()),
        ..empty()
    }),
("DescribeTargetGroupAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Attributes contains only Key and Value, without TargetGroupArn; requires identity using the request TargetGroupArn plus Key or enrichment of its parent target group to avoid cross-group collisions.".to_owned()),
        ..empty()
    }),
("DescribeTargetHealth", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TargetHealthDescriptions lacks TargetGroupArn and TargetType. Requires bound parent ARN plus Target.Id and nullable Target.Port, Target.AvailabilityZone, Target.QuicServerId in identity, retained parent TargetType for instance/ip/lambda/alb relationships, and nullable composite-key support; item-only identity cannot preserve memberships across groups.".to_owned()),
        ..empty()
    }),
("DescribeTrustStoreAssociations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TrustStoreAssociations returns only ResourceArn; requires request TrustStoreArn in association identity and relationship provenance, since ResourceArn identifies the associated resource rather than its trust-store association.".to_owned()),
        ..empty()
    }),
("DescribeTrustStoreRevocations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TrustStoreRevocations items contain TrustStoreArn and RevocationId, but RevocationId is modeled long; identity validation accepts string/integer/boolean only. Requires 64-bit scalar identity support for the composite TrustStoreArn and RevocationId key; content retrieval must remain denied.".to_owned()),
        ..empty()
    }),
("GetResourcePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Output contains only Policy and no ResourceArn. Requires request ResourceArn in singleton identity or parent trust-store policy enrichment; Policy text is mutable configuration, not a stable resource identity.".to_owned()),
        ..empty()
    }),
("GetTrustStoreCaCertificatesBundle", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetTrustStoreRevocationContent", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("AddListenerCertificates", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
