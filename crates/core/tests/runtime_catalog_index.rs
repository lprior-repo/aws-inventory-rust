use inventory_core::{
    catalog::runtime::{RelationshipPlan, RuntimeCatalog},
    inventory::Kind,
};

#[path = "support/base_catalog.rs"]
mod base_catalog;
use base_catalog::base;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_the_committed_catalog_when_each_plan_is_looked_up_then_the_first_linear_match_is_returned()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(base()?)?;
    assert!(!catalog.plans().is_empty());

    for plan in catalog.plans() {
        let (service, operation) = (plan.service_name(), plan.operation_name());
        let linear = catalog
            .plans()
            .iter()
            .find(|entry| entry.service_name() == service && entry.operation_name() == operation)
            .map(std::ptr::from_ref);

        let indexed = catalog.lookup(service, operation).map(std::ptr::from_ref);

        assert_eq!(indexed, linear, "{service}:{operation}");
    }
    Ok(())
}

#[test]
fn given_names_between_and_beyond_real_plans_when_looked_up_then_nothing_is_returned() -> TestResult
{
    let catalog = RuntimeCatalog::from_bytes(base()?)?;
    let plan = catalog.plans().first().ok_or("catalog has no plan")?;
    let longer = format!("{}x", plan.operation_name());

    assert!(catalog.lookup(plan.service_name(), &longer).is_none());
    assert!(catalog.lookup(plan.service_name(), "").is_none());
    assert!(catalog.lookup("", plan.operation_name()).is_none());
    assert!(catalog.lookup("zzzz-absent", "ListNothing").is_none());
    Ok(())
}

#[test]
fn given_the_committed_catalog_when_each_kind_name_path_is_read_then_the_first_linear_match_is_returned()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(base()?)?;
    assert!(!catalog.name_paths().is_empty());

    for entry in catalog.name_paths() {
        let linear = catalog
            .name_paths()
            .iter()
            .find(|candidate| candidate.kind == entry.kind)
            .map(std::ptr::from_ref);

        let indexed = catalog.name_path(&entry.kind).map(std::ptr::from_ref);

        assert_eq!(indexed, linear, "{}", entry.kind);
    }
    Ok(())
}

#[test]
fn given_the_committed_catalog_when_relations_are_read_by_source_kind_then_they_match_a_linear_filter_in_order()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(base()?)?;
    assert!(!catalog.relationships().is_empty());

    for relation in catalog.relationships() {
        let linear: Vec<*const RelationshipPlan> = catalog
            .relationships()
            .iter()
            .filter(|candidate| candidate.source_kind == relation.source_kind)
            .map(std::ptr::from_ref)
            .collect();

        let indexed: Vec<*const RelationshipPlan> = catalog
            .relationships_from(&relation.source_kind)
            .map(std::ptr::from_ref)
            .collect();

        assert_eq!(indexed, linear, "{}", relation.source_kind);
    }
    Ok(())
}

#[test]
fn given_a_kind_the_catalog_does_not_hold_when_read_then_no_name_path_and_no_relations_are_returned()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(base()?)?;
    let absent = Kind::parse("AWS::Fixture::Absent")?;
    assert!(
        catalog
            .relationships()
            .iter()
            .all(|relation| relation.source_kind != absent)
    );

    assert!(catalog.name_path(&absent).is_none());
    assert_eq!(catalog.relationships_from(&absent).count(), 0);
    Ok(())
}
