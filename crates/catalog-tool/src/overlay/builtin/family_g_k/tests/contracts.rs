use std::{collections::BTreeSet, fs, path::{Path, PathBuf}};

use crate::{
    classes::DenyList,
    overlay::{self, EffectiveClass, ItemSelector, OverlayClass, ReviewDisposition, ServiceOverlay},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models")
}

fn pinned_operations(service: &str) -> TestResult<BTreeSet<String>> {
    let mut operations = BTreeSet::new();
    let models = source().join("models").join(service).join("service");
    for file in crate::model_source::discover_models(&models)? {
        let value = crate::model_source::read_value(&file)?;
        let shapes = value.get("shapes").and_then(serde_json::Value::as_object)
            .ok_or("pinned model has no shapes")?;
        for (name, shape) in shapes {
            if shape.get("type").and_then(serde_json::Value::as_str) == Some("operation") {
                // Public service operation keys are short names.
                operations.insert(name.rsplit('#').next().ok_or("missing operation name")?.to_owned());
            }
        }
    }
    assert!(!operations.is_empty(), "{service}: no pinned operations");
    Ok(operations)
}

pub(super) fn assert_reviewed(policies: &[ServiceOverlay]) -> TestResult {
    // Given independent pinned models, compare operation keys rather than raw shape counts.
    for policy in policies {
        assert_eq!(policy.operations().keys().map(ToString::to_string).collect::<BTreeSet<_>>(),
            pinned_operations(policy.service())?, "{}", policy.service());
    }
    // When the public checker constructs the compiler's callable-operation handles.
    let checked = overlay::check_policies(&source(), policies, &DenyList::builtin())?;
    assert_eq!(checked.services().iter().map(|service| service.service()).collect::<BTreeSet<_>>(),
        policies.iter().map(ServiceOverlay::service).collect());
    for policy in policies {
        let service = checked.services().iter().find(|service| service.service() == policy.service())
            .ok_or("reviewed service missing")?;
        assert_eq!(service.decisions().iter().map(|decision| decision.operation()).collect::<BTreeSet<_>>(),
            policy.operations().keys().map(|name| name.as_str()).collect(), "{}", policy.service());
        let expected_callable = policy.operations().iter().filter_map(|(name, plan)| {
            matches!(plan.class, Some(OverlayClass::Enumerate | OverlayClass::Expand))
                .then_some(name.as_str())
        }).collect::<BTreeSet<_>>();
        assert_eq!(service.callable().iter().map(|operation| operation.operation()).collect::<BTreeSet<_>>(),
            expected_callable, "{}", policy.service());
        // Then every reviewed operation has the intended consumer-visible disposition;
        // denied/deferred operations never acquire callable handles.
        for (name, plan) in policy.operations() {
            let decision = service.decisions().iter().find(|decision| decision.operation() == name.as_str())
                .ok_or("reviewed operation missing")?;
            match (plan.class, decision.disposition()) {
                (Some(OverlayClass::Enumerate | OverlayClass::Expand), ReviewDisposition::Callable) => {}
                (Some(OverlayClass::Deny(_)), ReviewDisposition::Denied { reason }) => {
                    assert!(!reason.trim().is_empty(), "{}/{name}", policy.service());
                    assert_eq!(plan.identity, None);
                    assert_eq!(plan.items, None);
                }
                (Some(OverlayClass::Defer), ReviewDisposition::Deferred { reason }) => {
                    assert!(!reason.trim().is_empty(), "{}/{name}", policy.service());
                    assert!(!plan.defer_reason.as_deref().unwrap_or_default().trim().is_empty());
                    assert_eq!(plan.identity, None);
                    assert_eq!(plan.items, None);
                }
                actual => return Err(format!("{}/{name}: unexpected review {actual:?}", policy.service()).into()),
            }
        }
        for callable in service.callable() {
            let plan = callable.plan();
            let label = format!("{}/{}", policy.service(), callable.operation());
            if plan.endpoint_discovery {
                // Discovery reads one unpaginated root response and never emits a resource.
                assert_eq!(plan.identity, None, "{label}");
                assert_eq!(plan.items, Some(ItemSelector::Root), "{label}");
                assert_eq!(plan.paginator, Some(overlay::Paginator::None), "{label}");
            } else if let Some(tags) = &plan.tag_endpoint {
                // Tag reads enrich an existing parent, not an invented tag resource.
                assert_eq!(callable.class(), EffectiveClass::Expand, "{label}");
                assert_eq!(plan.identity, None, "{label}");
                assert_eq!(plan.items, Some(ItemSelector::Root), "{label}");
                assert!(!tags.parents.is_empty(), "{label}");
                for parent in &tags.parents {
                    assert!(expected_callable.contains(parent.operation.as_str()), "{label}: {}", parent.operation);
                }
                assert!(plan.scope.is_some(), "{label}");
                assert!(plan.paginator.is_some(), "{label}");
            } else {
                assert!(plan.identity.is_some(), "{label}");
                assert!(plan.scope.is_some(), "{label}");
                assert!(plan.items.is_some(), "{label}");
                assert!(plan.paginator.is_some(), "{label}");
            }
        }
    }
    assert_compiled(policies, &checked)?;
    Ok(())
}

fn assert_compiled(policies: &[ServiceOverlay], checked: &overlay::CheckedOverlays) -> TestResult {
    // Given complete, unmodified pinned service models and their real endpoint-rule cases.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pinned = source();
    let selected = tempfile::tempdir()?;
    for policy in policies {
        let directory = pinned.join("models").join(policy.service()).join("service");
        for file in crate::model_source::discover_models(&directory)? {
            let destination = selected.path().join(file.strip_prefix(&pinned)?);
            fs::create_dir_all(destination.parent().ok_or("model parent missing")?)?;
            fs::copy(file, destination)?;
        }
    }
    // When current production compilation is consumed in memory, never read the stale binary.
    let registry = crate::smithy::Registry::parse(&fs::read_to_string(root.join("catalog/field-ids.txt"))?)?;
    let base = crate::smithy::build_catalog(selected.path(), &registry, "gk-tz-pinned-regression")?;
    let kinds = crate::kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"), "/../../catalog/cloudcontrol-universal.json")))?;
    let endpoints = crate::endpoints::verified_endpoints(selected.path())?;
    let bytes = crate::runtime_catalog::augment(&base, checked, &kinds, &endpoints)?;
    let runtime = inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&bytes)?;
    // Then all reviewed callable keys survive numeric lowering, and noncallable keys do not.
    let expected = checked.services().iter().flat_map(|service| service.callable().iter())
        .map(|operation| (operation.service(), operation.operation())).collect::<BTreeSet<_>>();
    let actual = runtime.plans().iter().map(|plan| (plan.service_name(), plan.operation_name()))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    Ok(())
}
