use crate::overlay;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, MemberPath, OperationName, OverlayClass,
    Paginator, Scope, ServiceOverlay,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn operation<'a>(
    policies: &'a [ServiceOverlay],
    service: &str,
    name: &str,
) -> Result<&'a crate::overlay::OperationOverlay, Box<dyn std::error::Error>> {
    policies
        .iter()
        .find(|policy| policy.service() == service)
        .and_then(|policy| policy.operations().get(&OperationName::new(name).ok()?))
        .ok_or_else(|| format!("missing policy {service}/{name}").into())
}

#[test]
fn public_checker_accepts_the_real_ecr_parent_and_composite_image_plan() -> TestResult {
    // Given ECR's DescribeImages returns a list of ImageIds where each item carries
    // an ImageManifest that is not the resource itself but a reference to a manifest
    // in the registry.
    let policies = overlay::embedded()?;
    let plan = operation(&policies, "ecr", "DescribeImages")?;
    assert_eq!(plan.class, Some(OverlayClass::Expand));
    assert_eq!(plan.items, Some(ItemSelector::parse("imageDetails")?));
    assert_eq!(plan.scope, Some(Scope::Region));
    assert_eq!(
        plan.identity,
        Some(Identity::Key(vec![
            MemberPath::new("registryId")?,
            MemberPath::new("repositoryName")?,
            MemberPath::new("imageDigest")?,
        ]))
    );
    Ok(())
}

#[test]
fn search_domain_roots_partition_engine_types_without_changing_canonical_identities() -> TestResult
{
    let policies = overlay::embedded()?;
    let engine = MemberPath::new("EngineType")?;
    for (service, value) in [
        ("elasticsearch-service", "Elasticsearch"),
        ("opensearch", "OpenSearch"),
    ] {
        let plan = operation(&policies, service, "ListDomainNames")?;
        assert_eq!(
            plan.fixed.get(&engine),
            Some(&FixedValue::String(value.to_owned()))
        );
        assert_eq!(
            plan.identity,
            Some(Identity::Key(vec![MemberPath::new("DomainName")?]))
        );
    }
    Ok(())
}

#[test]
fn databrew_identity_uses_required_unique_names_not_optional_arns() -> TestResult {
    let policies = overlay::embedded()?;
    for name in ["ListJobs", "ListProjects", "ListRulesets", "ListSchedules"] {
        assert_eq!(
            operation(&policies, "databrew", name)?.identity,
            Some(Identity::Key(vec![MemberPath::new("Name")?]))
        );
    }
    Ok(())
}

#[test]
fn selected_scalar_helpers_preserve_strict_request_domains() -> TestResult {
    use super::support;
    // Given a scalar identity and a parent-selected scalar binding.
    let entry = support::inventory(
        "ListChildren",
        "names",
        support::key(&["$"])?,
        Paginator::None,
    )?;
    let bound = support::bound(entry.clone(), &[("parent", "ListParents", "$")])?;
    // Then only the selected side accepts the root marker.
    assert_eq!(
        bound.1.bind.get(&MemberPath::new("parent")?),
        Some(&Binding::From {
            operation: OperationName::new("ListParents")?,
            path: MemberPath::selected("$")?,
        })
    );
    assert_eq!(
        support::arn("$")?,
        Identity::Arn(MemberPath::selected("$")?)
    );
    assert_eq!(
        support::queried(entry.clone(), &["$"])?.1.queryable,
        vec![MemberPath::selected("$")?]
    );
    let error = overlay::NameError {
        value: "$".to_owned(),
    };
    assert_eq!(
        support::bound(entry.clone(), &[("$", "ListParents", "$")]),
        Err(error.clone())
    );
    assert_eq!(
        support::inherited(entry.clone(), "parent", "ListParents", "$"),
        Err(error.clone())
    );
    assert_eq!(support::fixed_integer(entry, "$", 10), Err(error.clone()));
    assert_eq!(support::correlated(&["$"], &["$"]), Err(error.clone()));
    assert_eq!(support::token("$", "nextToken"), Err(error.clone()));
    assert_eq!(support::token("nextToken", "$"), Err(error));
    Ok(())
}
