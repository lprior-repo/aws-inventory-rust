use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::classes::{DenyList, DenyReason};
use crate::overlay::{
    self, Binding, FixedValue, Identity, ItemSelector, MemberPath, OperationName, OperationOverlay,
    OverlayClass, Paginator, RequestRouting, Scope, ServiceOverlay, TokenPair,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn operation<'a>(
    policies: &'a [ServiceOverlay],
    service: &str,
    operation: &str,
) -> Result<&'a OperationOverlay, Box<dyn std::error::Error>> {
    policies
        .iter()
        .find(|policy| policy.service() == service)
        .ok_or_else(|| format!("missing service {service}"))?
        .operations()
        .get(&OperationName::new(operation)?)
        .ok_or_else(|| format!("missing operation {service}/{operation}").into())
}

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models")
}

fn model_files(root: &Path, files: &mut Vec<PathBuf>) -> TestResult {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            model_files(&entry.path(), files)?;
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(entry.path());
        }
    }
    Ok(())
}

#[test]
fn every_pinned_q_s_service_and_operation_has_an_explicit_decision() -> TestResult {
    // Given the independently pinned service tree, including services with no safe root.
    let policies = super::overlays()?;
    let excluded = ["rds", "route-53", "s3", "sfn", "sns", "sqs", "sts"];
    let mut expected_services = BTreeSet::new();
    for entry in fs::read_dir(source().join("models"))? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "non-UTF8 service name")?;
        if entry.file_type()?.is_dir()
            && name.as_str() >= "q"
            && name.as_str() < "t"
            && !excluded.contains(&name.as_str())
        {
            expected_services.insert(name);
        }
    }
    // When resolving the entire assigned family, no service disappears for lack of support.
    assert_eq!(
        policies
            .iter()
            .map(|policy| policy.service().to_owned())
            .collect::<BTreeSet<_>>(),
        expected_services
    );
    assert_eq!(policies.len(), 82);
    for policy in &policies {
        let mut files = Vec::new();
        model_files(
            &source()
                .join("models")
                .join(policy.service())
                .join("service"),
            &mut files,
        )?;
        let mut expected_operations = BTreeSet::new();
        for file in files {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(file)?)?;
            let shapes = value
                .get("shapes")
                .and_then(serde_json::Value::as_object)
                .ok_or("pinned model has no shapes")?;
            for (name, shape) in shapes {
                if shape.get("type").and_then(serde_json::Value::as_str) == Some("operation") {
                    expected_operations.insert(
                        name.rsplit('#')
                            .next()
                            .ok_or("missing shape name")?
                            .to_owned(),
                    );
                }
            }
        }
        // Then every actual modeled operation is inventoried, denied, or explicitly blocked.
        assert!(!expected_operations.is_empty(), "{}", policy.service());
        let actual = policy
            .operations()
            .keys()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        if actual != expected_operations {
            let in_model_not_overlay = expected_operations.difference(&actual);
            let in_overlay_not_model = actual.difference(&expected_operations);
            eprintln!(
                "sagemaker diff - in model not overlay: {:?}",
                in_model_not_overlay.collect::<Vec<_>>()
            );
            eprintln!(
                "sagemaker diff - in overlay not model: {:?}",
                in_overlay_not_model.collect::<Vec<_>>()
            );
        }
        assert_eq!(actual, expected_operations, "{}", policy.service());
        for decision in policy.operations().values() {
            assert!(decision.class.is_some());
            if decision.class == Some(OverlayClass::Defer) {
                assert!(
                    !decision
                        .defer_reason
                        .as_deref()
                        .unwrap_or_default()
                        .is_empty()
                );
                assert_eq!(decision.identity, None);
                assert_eq!(decision.items, None);
            }
        }
    }
    Ok(())
}

#[test]
fn pinned_model_checker_accepts_every_callable_q_s_contract() -> TestResult {
    // Given the real pinned models, not an echo of the policy data.
    let policies = super::overlays()?;
    // When typed policies are checked for request/output/type compatibility.
    let checked = overlay::check_policies(&source(), &policies, &DenyList::builtin())?;
    // Then all authored callable operations survive the safety/model boundary.
    let authored = policies
        .iter()
        .flat_map(|service| {
            service
                .operations()
                .iter()
                .filter(|(_, operation)| {
                    matches!(
                        operation.class,
                        Some(OverlayClass::Enumerate | OverlayClass::Expand)
                    )
                })
                .map(move |(operation, _)| (service.service().to_owned(), operation.to_string()))
        })
        .collect::<BTreeSet<_>>();
    let callable = checked
        .services()
        .iter()
        .flat_map(|service| {
            service.callable().iter().map(move |operation| {
                (
                    service.service().to_owned(),
                    operation.operation().to_owned(),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(callable, authored);
    // Then customer permissions and caller-owned applications retain their exact request domains.
    let permissions = operation(&policies, "ram", "ListPermissions")?;
    assert_eq!(
        permissions.fixed.get(&MemberPath::new("permissionType")?),
        Some(&FixedValue::String("CUSTOMER_MANAGED".to_owned()))
    );
    assert_eq!(
        permissions.fixed.get(&MemberPath::new("resourceOwner")?),
        Some(&FixedValue::String("SELF".to_owned()))
    );
    let applications = operation(&policies, "sso-admin", "ListApplications")?;
    assert_eq!(
        applications
            .bind
            .get(&MemberPath::new("Filter.ApplicationAccount")?),
        Some(&Binding::Account)
    );
    assert_eq!(
        applications.identity,
        Some(Identity::Arn(MemberPath::new("ApplicationArn")?))
    );
    // Both optional safety-rule arms remain callable, each using its exact returned ARN.
    let rules = operation(
        &policies,
        "route53-recovery-control-config",
        "ListSafetyRules",
    )?;
    let arms = ["ASSERTION.SafetyRuleArn", "GATING.SafetyRuleArn"]
        .into_iter()
        .map(|field| {
            let path = MemberPath::new(field)?;
            overlay::IdentityBranch::new(
                overlay::IdentityPredicate::Present(path.clone()),
                overlay::IdentityLeaf::Arn(path),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        rules.identity,
        Some(Identity::Choice(overlay::IdentityChoice::Discriminated(
            arms
        )))
    );
    assert_eq!(
        rules.bind.get(&MemberPath::new("ControlPanelArn")?),
        Some(&Binding::From {
            operation: OperationName::new("ListControlPanels")?,
            path: MemberPath::new("ControlPanelArn")?,
        })
    );
    Ok(())
}

#[test]
fn secret_content_billed_and_mutating_calls_cannot_be_authorized_by_projection() -> TestResult {
    // Given whole-response secret/content and billable service APIs.
    let policies = super::overlays()?;
    for (service, name, reason) in [
        ("secrets-manager", "GetSecretValue", DenyReason::Secret),
        ("ssm", "GetParameter", DenyReason::Content),
        ("sagemaker", "ListAppImageConfigs", DenyReason::Secret),
        ("sagemaker-runtime", "InvokeEndpoint", DenyReason::Paid),
        ("rds-data", "ExecuteStatement", DenyReason::Paid),
        ("ses", "SendEmail", DenyReason::Mutating),
    ] {
        // When resolving the actual embedded operation decision.
        let decision = operation(&policies, service, name)?;
        // Then no item, identity or projection accidentally turns the denied call executable.
        assert_eq!(
            decision.class,
            Some(OverlayClass::Deny(reason)),
            "{service}/{name}"
        );
        assert_eq!(decision.identity, None);
        assert_eq!(decision.items, None);
        assert!(decision.queryable.is_empty());
    }
    Ok(())
}

#[test]
fn secrets_inventory_includes_planned_deletion_without_fetching_secret_values() -> TestResult {
    let policies = super::overlays()?;
    // Given metadata-only ListSecrets, including resources pending deletion.
    let decision = operation(&policies, "secrets-manager", "ListSecrets")?;
    assert_eq!(decision.class, Some(OverlayClass::Enumerate));
    assert_eq!(
        decision.identity,
        Some(Identity::Arn(MemberPath::selected("ARN")?))
    );
    assert_eq!(
        decision
            .fixed
            .get(&MemberPath::selected("IncludePlannedDeletion")?),
        Some(&FixedValue::Boolean(true))
    );
    // Then the full token chain is preserved, rather than silently collecting one page.
    assert_eq!(
        decision.paginator,
        Some(Paginator::Token(TokenPair {
            input: MemberPath::selected("NextToken")?,
            output: MemberPath::selected("NextToken")?,
            optional: false,
        }))
    );
    Ok(())
}

#[test]
fn s3_control_all_data_sources_and_global_mrap_routing_are_explicit() -> TestResult {
    let policies = super::overlays()?;
    // Given pinned hostPrefix {AccountId}. and AccountId hostLabel/header traits.
    let decision = operation(&policies, "s3-control", "ListAccessPoints")?;
    assert_eq!(decision.class, Some(OverlayClass::Enumerate));
    assert_eq!(
        decision.bind.get(&MemberPath::selected("AccountId")?),
        Some(&Binding::Account)
    );
    assert_eq!(decision.scope, Some(Scope::Region));
    assert_eq!(
        decision.fixed.get(&MemberPath::selected("DataSourceType")?),
        Some(&FixedValue::String("ALL".to_owned())),
    );
    // Then the supported modeled prefix is used, but MRAP fixed us-west-2 routing is explicit.
    let mrap = operation(&policies, "s3-control", "ListMultiRegionAccessPoints")?;
    assert_eq!(mrap.class, Some(OverlayClass::Enumerate));
    assert_eq!(mrap.scope, Some(Scope::Account));
    assert_eq!(
        mrap.routing,
        RequestRouting::FixedRegion {
            region: "us-west-2".to_owned()
        }
    );
    let directory = operation(
        &policies,
        "s3-control",
        "ListAccessPointsForDirectoryBuckets",
    )?;
    assert_eq!(directory.routing, RequestRouting::S3ExpressControl);
    Ok(())
}

#[test]
fn supply_chain_dataset_expansion_keeps_both_identifiers_on_one_parent_row() -> TestResult {
    let policies = super::overlays()?;
    // Given namespaces from potentially different Supply Chain instances.
    let decision = operation(&policies, "supplychain", "ListDataLakeDatasets")?;
    assert_eq!(decision.class, Some(OverlayClass::Expand));
    // Then instance and namespace come from the same namespace item, never a Cartesian product.
    for (member, source) in [("instanceId", "instanceId"), ("namespace", "name")] {
        assert_eq!(
            decision.bind.get(&MemberPath::selected(member)?),
            Some(&Binding::From {
                operation: OperationName::new("ListDataLakeNamespaces")?,
                path: MemberPath::selected(source)?,
            })
        );
    }
    assert_eq!(
        decision.identity,
        Some(Identity::Arn(MemberPath::selected("arn")?))
    );
    Ok(())
}

#[test]
fn singleton_local_index_and_scalar_permission_set_arns_use_real_items() -> TestResult {
    let policies = super::overlays()?;
    // Given GetIndex explicitly describes the Region in which it was invoked.
    let index = operation(&policies, "resource-explorer-2", "GetIndex")?;
    assert_eq!(index.items, Some(ItemSelector::Root));
    assert_eq!(
        index.identity,
        Some(Identity::Arn(MemberPath::selected("Arn")?))
    );
    // Given PermissionSets is a list of ARN strings, not structures with an invented field.
    let permissions = operation(&policies, "sso-admin", "ListPermissionSets")?;
    assert_eq!(
        permissions.identity,
        Some(Identity::Arn(MemberPath::selected("$")?))
    );
    assert_eq!(
        permissions.bind.get(&MemberPath::selected("InstanceArn")?),
        Some(&Binding::From {
            operation: OperationName::new("ListInstances")?,
            path: MemberPath::selected("InstanceArn")?,
        })
    );
    Ok(())
}

#[test]
fn incomplete_union_and_foreign_scope_enumerations_are_explicitly_noncallable() -> TestResult {
    let policies = super::overlays()?;
    // Given APIs whose apparent root would omit variants or misattribute other Regions.
    for (service, name, reason) in [
        ("servicediscovery", "ListNamespaces", "DNS_PUBLIC"),
        ("resource-explorer-2", "ListIndexes", "per-item"),
    ] {
        // Then clients see the actual blocker rather than an unsupported placeholder or partial page.
        let decision = operation(&policies, service, name)?;
        assert_eq!(decision.class, Some(OverlayClass::Defer));
        assert_eq!(decision.items, None);
        assert!(
            decision
                .defer_reason
                .as_deref()
                .unwrap_or_default()
                .contains(reason),
            "{service}/{name}"
        );
    }
    Ok(())
}

#[test]
fn regional_lakes_and_all_retention_types_have_complete_native_request_contracts() -> TestResult {
    let policies = super::overlays()?;
    let lakes = operation(&policies, "securitylake", "ListDataLakes")?;
    assert_eq!(lakes.class, Some(OverlayClass::Enumerate));
    assert_eq!(lakes.scope, Some(Scope::Region));
    assert_eq!(
        lakes.bind.get(&MemberPath::selected("regions")?),
        Some(&Binding::RegionList)
    );
    let rules = operation(&policies, "rbin", "ListRules")?;
    assert_eq!(rules.class, Some(OverlayClass::Enumerate));
    assert_eq!(
        rules
            .branches
            .iter()
            .map(|branch| branch.discriminator.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["EBS_VOLUME", "EBS_SNAPSHOT", "EC2_IMAGE"])
    );
    Ok(())
}

#[test]
fn s3_control_safe_details_use_reviewed_parent_identifiers_and_returned_arns() -> TestResult {
    let policies = super::overlays()?;
    for (name, parent, input, source, arn) in [
        (
            "GetStorageLensGroup",
            "ListStorageLensGroups",
            "Name",
            "Name",
            "StorageLensGroupArn",
        ),
        (
            "GetAccessGrant",
            "ListAccessGrants",
            "AccessGrantId",
            "AccessGrantId",
            "AccessGrantArn",
        ),
        (
            "GetAccessGrantsLocation",
            "ListAccessGrantsLocations",
            "AccessGrantsLocationId",
            "AccessGrantsLocationId",
            "AccessGrantsLocationArn",
        ),
    ] {
        let detail = operation(&policies, "s3-control", name)?;
        assert_eq!(detail.class, Some(OverlayClass::Expand));
        assert_eq!(detail.scope, Some(Scope::Region));
        assert_eq!(
            detail.identity,
            Some(Identity::Arn(MemberPath::selected(arn)?))
        );
        assert_eq!(
            detail.bind.get(&MemberPath::selected("AccountId")?),
            Some(&Binding::Account)
        );
        assert_eq!(
            detail.bind.get(&MemberPath::selected(input)?),
            Some(&Binding::From {
                operation: OperationName::new(parent)?,
                path: MemberPath::selected(source)?,
            })
        );
    }
    Ok(())
}
