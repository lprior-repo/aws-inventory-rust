use catalog_tool::{
    classes::DenyReason,
    overlay::{self, FixedValue, Identity, ItemSelector, OverlayClass, Paginator},
};
use std::error::Error;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn operation<'a>(
    overlays: &'a [overlay::ServiceOverlay],
    service_name: &str,
    operation_name: &str,
) -> TestResult<&'a overlay::OperationOverlay> {
    overlays
        .iter()
        .find(|service| service.service() == service_name)
        .and_then(|service| service.operations().get(operation_name))
        .ok_or_else(|| format!("missing {service_name}.{operation_name}").into())
}

#[test]
fn ec2_instances_remain_deferred_until_owner_context_is_preserved() -> TestResult {
    let overlays = overlay::embedded()?;
    let plan = operation(&overlays, "ec2", "DescribeInstances")?;

    assert_eq!(plan.class, Some(OverlayClass::Defer));
    let reason = plan.defer_reason.as_deref().ok_or("defer reason missing")?;
    assert!(reason.contains("Reservations[].Instances[]"));
    assert!(reason.contains("Reservations[].OwnerId"));
    Ok(())
}

#[test]
fn task_definition_plan_projects_status_product_page_bound_and_read_only_rows() -> TestResult {
    let overlays = overlay::embedded()?;
    let list = operation(&overlays, "ecs", "ListTaskDefinitions")?;

    assert_eq!(list.class, Some(OverlayClass::Enumerate));
    assert_eq!(list.scope, Some(overlay::Scope::Region));
    assert_eq!(list.items, Some(ItemSelector::parse("taskDefinitionArns")?));
    let arn_identity = Identity::Arn(overlay::MemberPath::selected("$")?);
    assert_eq!(list.identity.as_ref(), Some(&arn_identity));
    assert_eq!(list.page_size, Some(100));
    assert_eq!(
        list.fixed.get(&overlay::MemberPath::new("sort")?),
        Some(&FixedValue::String("ASC".to_owned()))
    );
    assert_eq!(list.branches.len(), STATUSES.len());
    assert_eq!(
        list.branches
            .iter()
            .map(|branch| branch.discriminator.as_str())
            .collect::<Vec<_>>(),
        STATUSES
    );
    let status_path = overlay::MemberPath::new("status")?;
    for (branch, status) in list.branches.iter().zip(STATUSES) {
        assert_eq!(
            branch.fixed.get(&status_path),
            Some(&FixedValue::String(status.to_owned()))
        );
        assert_eq!(branch.scope, overlay::Scope::Region);
    }
    assert!(matches!(
        &list.paginator,
        Some(Paginator::Token(token))
            if token.input.as_str() == "nextToken" && token.output.as_str() == "nextToken"
    ));

    let detail = operation(&overlays, "ecs", "DescribeTaskDefinition")?;
    assert_eq!(detail.class, Some(OverlayClass::Deny(DenyReason::Content)));
    Ok(())
}


const STATUSES: [&str; 3] = ["ACTIVE", "INACTIVE", "DELETE_IN_PROGRESS"];
