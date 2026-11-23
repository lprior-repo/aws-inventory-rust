use catalog_tool as fixture_catalog;
#[path = "../../src/overlay/builtin/family_a_c/consumer_fixture.rs"]
mod model;
use inventory_core::catalog::runtime::{BindingSource, RuntimeCatalog};

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;
pub(super) struct Case<'a> {
    pub service: &'a str,
    pub date: &'a str,
    pub operation: &'a str,
    pub parent: &'a str,
    pub input: &'a str,
    pub output: &'a str,
}

pub(super) fn verify(case: Case<'_>) -> TestResult {
    let compiled = model::compile(case.service, case.date, &[case.parent, case.operation])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup(case.service, case.operation)
        .ok_or("child missing")?;
    let input = compiled.fields.id(case.input).ok_or("input missing")?;
    let output = compiled.fields.id(case.output).ok_or("output missing")?;
    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.path.as_ref() == [input])
        .ok_or("required parent binding missing")?;
    assert!(
        matches!(&binding.source, BindingSource::Operation { service, operation, path }
        if service.as_ref() == case.service && operation.as_ref() == case.parent && path.as_ref() == [output])
    );
    Ok(())
}
