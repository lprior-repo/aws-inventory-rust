#[path = "../family_g_k/tests/contracts.rs"]
mod gz_contracts;

use crate::{
    classes::DenyReason,
    overlay::{self, Binding, Identity, MemberPath, OperationName, OperationOverlay, OverlayClass,
        Paginator, Scope, ServiceOverlay, TokenPair},
};
use std::collections::BTreeSet;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn policy<'a>(
    services: &'a [ServiceOverlay],
    service: &str,
    operation: &str,
) -> Result<&'a OperationOverlay, Box<dyn std::error::Error>> {
    let service = services.iter().find(|candidate| candidate.service == service)
        .ok_or_else(|| std::io::Error::other(format!("missing service {service}")))?;
    service.operations.get(&OperationName::new(operation)?)
        .ok_or_else(|| std::io::Error::other(format!("missing operation {operation}")).into())
}

#[test]
fn embedded_registry_exposes_every_pinned_t_z_namespace_and_explicit_operation() -> TestResult {
    // Given: all 28 pinned non-first-wave namespaces in this partition.
    let expected = ["taxsettings", "textract", "timestream-influxdb", "timestream-query",
        "timestream-write", "transcribe", "transcribe-streaming", "transfer", "translate",
        "trustedadvisor", "uxc", "verifiedpermissions", "voice-id", "vpc-lattice", "waf",
        "waf-regional", "wafv2", "wellarchitected", "wickr", "wisdom", "workdocs", "workmail",
        "workmailmessageflow", "workspaces", "workspaces-instances", "workspaces-thin-client",
        "workspaces-web", "xray"].into_iter().collect::<BTreeSet<_>>();
    // When: the application's public embedded registry is loaded.
    let services = overlay::embedded()?;
    let selected = services.iter().filter(|service| service.service.as_str() >= "t")
        .collect::<Vec<_>>();
    // Then: no namespace or operation in the pinned partition silently disappears.
    let actual = selected.iter().map(|service| service.service.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    gz_contracts::assert_reviewed(&selected.into_iter().cloned().collect::<Vec<_>>())?;
    Ok(())
}

#[test]
fn secret_or_content_bearing_calls_remain_denied_before_any_projection() -> TestResult {
    // Given: calls with documented credential, document, transcript, or trace payloads.
    let cases = [
        ("wickr", "GetOidcInfo", DenyReason::Secret),
        ("wickr", "GetOpentdfConfig", DenyReason::Secret),
        ("workspaces-web", "GetIdentityProvider", DenyReason::Secret),
        ("workspaces-thin-client", "GetEnvironment", DenyReason::Secret),
        ("workspaces-thin-client", "ListEnvironments", DenyReason::Secret),
        ("wafv2", "GetDecryptedAPIKey", DenyReason::Secret),
        ("wafv2", "ListAPIKeys", DenyReason::Secret),
        ("textract", "GetDocumentAnalysis", DenyReason::Content),
        ("transcribe", "GetVocabulary", DenyReason::Content),
        ("translate", "GetTerminology", DenyReason::Content),
        ("timestream-query", "Query", DenyReason::Content),
        ("timestream-query", "DescribeScheduledQuery", DenyReason::Content),
        ("workdocs", "DescribeComments", DenyReason::Content),
        ("workmailmessageflow", "GetRawMessageContent", DenyReason::Content),
        ("xray", "ListRetrievedTraces", DenyReason::Content),
    ];
    // When: callers inspect the public reviewed policy rather than infer from read names.
    let services = overlay::embedded()?;
    // Then: every whole call is denied, not allowed by hiding its sensitive fields.
    for (service, operation, reason) in cases {
        let operation = policy(&services, service, operation)?;
        assert_eq!(operation.class, Some(OverlayClass::Deny(reason)));
        assert_eq!(operation.items, None);
        assert_eq!(operation.identity, None);
    }
    Ok(())
}

#[test]
fn classic_waf_uses_documented_markers_and_distinguishes_global_from_regional() -> TestResult {
    // Given: Classic WAF APIs have explicit marker fields without paginator traits.
    let services = overlay::embedded()?;
    // When: both endpoint scopes expose their IP set inventory.
    let global = policy(&services, "waf", "ListIPSets")?;
    let regional = policy(&services, "waf-regional", "ListIPSets")?;
    // Then: all pages are requested and global resources are not duplicated per region.
    let paginator = Some(Paginator::Token(TokenPair {
        input: MemberPath::new("NextMarker")?, output: MemberPath::new("NextMarker")?,
        optional: false,
    }));
    assert_eq!(global.paginator, paginator);
    assert_eq!(regional.paginator, paginator);
    assert_eq!(global.scope, Some(Scope::Account));
    assert_eq!(regional.scope, Some(Scope::Region));
    assert_eq!(global.identity, Some(Identity::Key(vec![MemberPath::new("IPSetId")?])));
    assert_eq!(global.fixed.get(&MemberPath::new("Limit")?),
        Some(&overlay::FixedValue::Integer(100)));
    Ok(())
}

#[test]
fn bound_adapter_details_retain_both_version_identifiers_and_application_owner_is_account() -> TestResult {
    // Given: adapter versions need two input identifiers, and applications can be vendor-owned.
    let services = overlay::embedded()?;
    // When: the public registry supplies the executable detail and enumeration policies.
    let adapter = policy(&services, "textract", "GetAdapterVersion")?;
    let applications = policy(&services, "workspaces", "DescribeApplications")?;
    // Then: the version is not queried with a sibling's ID and vendor apps are excluded.
    assert_eq!(adapter.class, Some(OverlayClass::Expand));
    for field in ["AdapterId", "AdapterVersion"] {
        assert_eq!(adapter.bind.get(&MemberPath::new(field)?), Some(&Binding::From {
            operation: OperationName::new("ListAdapterVersions")?, path: MemberPath::new(field)?,
        }));
    }
    assert_eq!(adapter.tags, Some(MemberPath::new("Tags")?));
    assert_eq!(applications.class, Some(OverlayClass::Enumerate));
    assert_eq!(applications.bind.get(&MemberPath::new("Owner")?), Some(&Binding::Account));
    Ok(())
}

#[test]
fn required_endpoint_discovery_and_both_waf_scopes_are_executable_typed_plans() -> TestResult {
    let services = overlay::embedded()?;
    for (service, operation) in [
        ("timestream-query", "ListScheduledQueries"),
        ("timestream-write", "ListDatabases"),
        ("timestream-write", "ListTables"),
    ] {
        let plan = policy(&services, service, operation)?;
        assert!(matches!(plan.class, Some(OverlayClass::Enumerate | OverlayClass::Expand)));
        assert!(matches!(&plan.routing, overlay::RequestRouting::Discovered { operation, .. } if operation.as_str() == "DescribeEndpoints"));
        assert!(policy(&services, service, "DescribeEndpoints")?.endpoint_discovery);
    }
    let waf = policy(&services, "wafv2", "ListWebACLs")?;
    assert_eq!(waf.class, Some(OverlayClass::Enumerate));
    let scope = overlay::MemberPath::selected("Scope")?;
    let scopes = waf.branches.iter().map(|branch| branch.fixed.get(&scope)).collect::<Vec<_>>();
    assert_eq!(scopes, [
        Some(&overlay::FixedValue::String("REGIONAL".to_owned())),
        Some(&overlay::FixedValue::String("CLOUDFRONT".to_owned())),
    ]);
    assert_eq!(waf.branches.last().ok_or("CloudFront branch absent")?.request_region.as_deref(), Some("us-east-1"));
    for operation in ["ListTagsForResource", "GetLoggingConfiguration"] {
        let plan = policy(&services, "wafv2", operation)?;
        assert_eq!(plan.branches.iter().map(|branch| (
            branch.discriminator.as_str(), branch.scope, branch.request_region.as_deref(),
        )).collect::<Vec<_>>(), [
            ("REGIONAL", Scope::Region, None),
            ("CLOUDFRONT", Scope::Account, Some("us-east-1")),
        ]);
        assert!(plan.branches.iter().all(|branch| branch.fixed.is_empty()));
    }
    Ok(())
}

#[test]
fn agreement_details_bind_both_identifiers_from_the_same_list_item() -> TestResult {
    // Given: agreements retain their server identity in each paginated list row.
    let services = overlay::embedded()?;
    let list = policy(&services, "transfer", "ListAgreements")?;
    assert_eq!(list.paginator, Some(Paginator::Token(TokenPair {
        input: MemberPath::selected("NextToken")?,
        output: MemberPath::selected("NextToken")?,
        optional: false,
    })));
    // When: safe detail configuration is requested, both bindings use that row.
    let detail = policy(&services, "transfer", "DescribeAgreement")?;
    assert_eq!(detail.class, Some(OverlayClass::Expand));
    assert_eq!(detail.bind.len(), 2);
    for field in ["AgreementId", "ServerId"] {
        assert_eq!(detail.bind.get(&MemberPath::selected(field)?), Some(&Binding::From {
            operation: OperationName::new("ListAgreements")?,
            path: MemberPath::selected(field)?,
        }));
    }
    // Then: canonical ARN, tags and safe detail-only fields remain collectable.
    assert_eq!(detail.identity, Some(Identity::Arn(MemberPath::selected("Arn")?)));
    assert_eq!(detail.tags, Some(MemberPath::selected("Tags")?));
    for field in ["BaseDirectory", "AccessRole", "LocalProfileId", "PartnerProfileId"] {
        assert!(detail.queryable.contains(&MemberPath::selected(field)?));
    }
    Ok(())
}
