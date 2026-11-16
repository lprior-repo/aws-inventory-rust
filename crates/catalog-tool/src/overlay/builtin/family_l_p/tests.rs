use super::overlays;
use crate::{
    classes::DenyReason,
    overlay::{Binding, FixedValue, ItemSelector, OperationOverlay, OverlayClass, Paginator, Scope},
};
use std::{collections::BTreeSet, error::Error, fs, path::Path};

type TestResult = Result<(), Box<dyn Error>>;

pub(super) fn operation(service: &str, operation: &str) -> Result<OperationOverlay, Box<dyn Error>> {
    let services = overlays()?;
    let selected = services.iter().find(|entry| entry.service() == service)
        .ok_or("service is missing")?;
    selected.operations().iter().find(|(name, _)| name.as_str() == operation)
        .map(|(_, policy)| policy.clone()).ok_or_else(|| "operation is missing".into())
}

fn pinned_operations(root: &Path, service: &str) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let versions = fs::read_dir(root.join(service).join("service"))?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(versions.len(), 1, "pinned service has changed versions: {service}");
    let version = versions.first().ok_or("missing version directory")?;
    let files = fs::read_dir(version.path())?.collect::<Result<Vec<_>, _>>()?;
    let models = files.iter().filter(|file| file.path().extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    assert_eq!(models.len(), 1, "pinned service has changed model count: {service}");
    let file = models.first().ok_or("missing Smithy model")?;
    let model: serde_json::Value = serde_json::from_slice(&fs::read(file.path())?)?;
    let shapes = model.get("shapes").and_then(serde_json::Value::as_object)
        .ok_or("missing Smithy shapes")?;
    Ok(shapes.iter().filter(|(_, shape)| shape.get("type").and_then(serde_json::Value::as_str)
        == Some("operation"))
        .filter_map(|(name, _)| name.split_once('#').map(|(_, local)| local.to_owned())).collect())
}

#[test]
fn given_pinned_models_when_policies_load_then_every_partition_operation_has_a_disposition() -> TestResult {
    // Given: the actual pinned Smithy model tree, independently of the authored policy tables.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models/models");
    let mut expected = BTreeSet::new();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let name = entry.file_name().into_string().map_err(|_| "non-UTF8 model directory")?;
        if name.as_str() >= "l" && name.as_str() < "q" && name != "lambda" && name != "organizations" {
            expected.insert(name);
        }
    }
    // When: the real typed policies are constructed.
    let policies = overlays()?;
    // Then: no service or operation can silently disappear, including unsafe and blocked calls.
    let actual = policies.iter().map(|service| service.service().to_owned()).collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    assert_eq!(policies.len(), 93);
    for policy in &policies {
        let actual = policy.operations().keys().map(|name| name.as_str().to_owned()).collect();
        assert_eq!(pinned_operations(&root, policy.service())?, actual, "{}", policy.service());
        for operation in policy.operations().values() {
            assert_ne!(operation.class, None);
            if operation.class == Some(OverlayClass::Defer) {
                assert_eq!(operation.defer_reason.as_ref().is_some_and(|reason| !reason.is_empty()), true);
            }
        }
    }
    assert_eq!(policies.iter().map(|service| service.operations().len()).sum::<usize>(), 3429);
    Ok(())
}

#[test]
fn given_credential_bearing_responses_when_reviewed_then_the_whole_call_is_denied() -> TestResult {
    // Given: models whose response bodies include credentials, arbitrary environment, or private content.
    let cases = [
        ("lightsail", "GetRelationalDatabases"),
        ("lightsail", "GetContainerServices"),
        ("mediapackage", "ListChannels"),
        ("mediaconvert", "ListJobTemplates"),
        ("mwaa", "GetEnvironment"),
        ("pca-connector-scep", "GetChallengePassword"),
    ];
    // When/Then: projection does not turn any such operation into an authorized inventory read.
    for (service, name) in cases {
        let policy = operation(service, name)?;
        assert_eq!(policy.class, Some(OverlayClass::Deny(DenyReason::Secret)), "{service}/{name}");
        assert_eq!(policy.items, None);
        assert_eq!(policy.identity, None);
    }
    Ok(())
}

#[test]
fn given_nested_pinpoint_items_when_enumerated_then_continuation_uses_the_nested_response_token() -> TestResult {
    // Given/When: Pinpoint's nonstandard wrapper response, not a guessed root-level Item list.
    let policy = operation("pinpoint", "GetApps")?;
    // Then: selector and paginator agree with ApplicationsResponse in the pinned model.
    assert_eq!(policy.items, Some(ItemSelector::parse("ApplicationsResponse.Item")?));
    let Some(Paginator::Token(pair)) = policy.paginator else { return Err("missing paginator".into()); };
    assert_eq!(pair.input.as_str(), "Token");
    assert_eq!(pair.output.as_str(), "ApplicationsResponse.NextToken");
    Ok(())
}

#[test]
fn given_channel_discovery_when_listing_origin_endpoints_then_both_inputs_bind_to_the_same_parent() -> TestResult {
    // Given/When: channel summaries carry both the channel group and channel name.
    let policy = operation("mediapackagev2", "ListOriginEndpoints")?;
    // Then: neither required input is guessed, omitted, or cross-joined from a different parent.
    assert_eq!(policy.class, Some(OverlayClass::Expand));
    assert_eq!(policy.bind.len(), 2);
    for (input, member) in [("ChannelGroupName", "ChannelGroupName"), ("ChannelName", "ChannelName")] {
        let binding = policy.bind.get(&crate::overlay::MemberPath::new(input)?).ok_or("missing binding")?;
        let Binding::From { operation, path } = binding else { return Err("wrong binding kind".into()); };
        assert_eq!(operation.as_str(), "ListChannels");
        assert_eq!(path.as_str(), member);
    }
    Ok(())
}

#[test]
fn given_summary_only_certificate_read_when_enumerated_then_expanded_details_are_explicitly_disabled() -> TestResult {
    // Given/When: certificates have a documented summary-only request switch.
    let policy = operation("lightsail", "GetCertificates")?;
    // Then: the request is fixed to summary semantics; the response projection is not the safety gate.
    assert_eq!(policy.fixed.get(&crate::overlay::MemberPath::new("includeCertificateDetails")?),
        Some(&FixedValue::Boolean(false)));
    assert_eq!(policy.class, Some(OverlayClass::Enumerate));
    Ok(())
}

#[test]
fn given_safe_metadata_resources_when_loaded_then_all_callable_plans_have_complete_execution_fields() -> TestResult {
    // Given/When: actual typed metadata policies across the whole family.
    let policies = overlays()?;
    // Then: there is no prefix-only authorization with a missing selector, identity, scope or paginator.
    for service in &policies {
        for policy in service.operations().values().filter(|policy| matches!(policy.class,
            Some(OverlayClass::Enumerate | OverlayClass::Expand))) {
            assert_eq!(policy.items.is_some(), true);
            assert_eq!(policy.scope.is_some(), true);
            assert_eq!(policy.paginator.is_some(), true);
            if let Some(endpoint) = &policy.tag_endpoint {
                assert!(!endpoint.parents.is_empty());
                assert!(policy.identity.is_none());
            } else {
                assert_eq!(policy.identity.is_some(), true);
                assert_eq!(policy.queryable.is_empty(), false);
            }
        }
    }
    assert_eq!(operation("networkmanager", "DescribeGlobalNetworks")?.scope, Some(Scope::Account));
    assert_eq!(operation("migrationhub-config", "DescribeHomeRegionControls")?.scope, Some(Scope::HomeRegion));
    Ok(())
}

#[test]
fn given_discovered_lightsail_disk_when_described_then_the_read_uses_its_name_and_retains_arn_identity() -> TestResult {
    // Given: the independent seed is CreateDisk(diskName, availabilityZone, sizeInGb).
    // When: its discovered metadata is used to request the singleton detail view.
    let policy = operation("lightsail", "GetDisk")?;
    // Then: a real detail call exists, not just an unchanged list response with a new label.
    assert_eq!(policy.class, Some(OverlayClass::Expand));
    assert_eq!(policy.items, Some(ItemSelector::parse("disk")?));
    assert_eq!(policy.paginator, Some(Paginator::None));
    let binding = policy.bind.get(&crate::overlay::MemberPath::new("diskName")?)
        .ok_or("missing disk parent binding")?;
    let Binding::From { operation, path } = binding else { return Err("wrong binding kind".into()); };
    assert_eq!(operation.as_str(), "GetDisks");
    assert_eq!(path.as_str(), "name");
    assert_eq!(policy.identity, Some(crate::overlay::Identity::Arn(
        crate::overlay::MemberPath::new("arn")?,
    )));
    Ok(())
}
