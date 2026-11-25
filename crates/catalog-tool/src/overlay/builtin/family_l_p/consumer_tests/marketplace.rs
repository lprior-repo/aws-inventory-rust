use super::*;

#[test]
fn given_documented_required_agreement_type_when_pinned_search_compiles_then_filter_is_model_shaped()
-> TestResult {
    // Given: filters is modeled optional, but its documentation requires AgreementType.
    let bytes = compile_pinned(
        "marketplace-agreement",
        "2020-03-01",
        &["SearchAgreements"],
        "AWS::ApiOnly::MarketplaceAgreement",
    )?;
    // When: the checked overlay is encoded and loaded through the runtime catalog.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("marketplace-agreement", "SearchAgreements")
        .ok_or("agreement search missing")?;
    let filters = field(&runtime, "marketplace-agreement", "filters")?;
    let binding = plan
        .bindings()
        .iter()
        .find(|binding| binding.path.as_ref() == [filters])
        .ok_or("agreement type filter absent")?;
    // Then: lowercase modeled members retain the mandatory type without narrowing party/status.
    assert_eq!(binding.source, expected_filter(&runtime)?);
    Ok(())
}

fn expected_filter(runtime: &RuntimeCatalog<'_>) -> TestResult<BindingSource> {
    let name = field(runtime, "marketplace-agreement", "name")?;
    let values = field(runtime, "marketplace-agreement", "values")?;
    Ok(BindingSource::Fixed(BindingValue::List(
        vec![BindingValue::Object(
            BTreeMap::from([
                (
                    Key::Field(name),
                    BindingValue::String("AgreementType".to_owned()),
                ),
                (
                    Key::Field(values),
                    BindingValue::Strings(vec!["PurchaseAgreement".to_owned()].into_boxed_slice()),
                ),
            ])
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        )]
        .into_boxed_slice(),
    )))
}
