use super::*;

#[test]
fn compiled_search_sends_purchase_agreement_filter_for_each_party() -> TestResult {
    let bytes = compile_pinned(
        "marketplace-agreement",
        "2020-03-01",
        &["SearchAgreements"],
        "AWS::ApiOnly::MarketplaceAgreement",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("marketplace-agreement", "SearchAgreements")
        .ok_or("agreement search missing")?;
    let filters = field(&runtime, "marketplace-agreement", "filters")?;
    let name = field(&runtime, "marketplace-agreement", "name")?;
    let values = field(&runtime, "marketplace-agreement", "values")?;
    let filter = |key: &str, value: &str| {
        BindingValue::Object(
            BTreeMap::from([
                (Key::Field(name), BindingValue::String(key.to_owned())),
                (
                    Key::Field(values),
                    BindingValue::Strings(vec![value.to_owned()].into_boxed_slice()),
                ),
            ])
            .into_iter()
            .collect(),
        )
    };
    assert_eq!(plan.branches().len(), 2);
    for (id, party) in [("proposer", "Proposer"), ("acceptor", "Acceptor")] {
        let branch = plan
            .branches()
            .iter()
            .find(|branch| branch.discriminator == id)
            .ok_or("party branch absent")?;
        let mut bindings = plan
            .effective_bindings(Some(branch))
            .filter(|binding| binding.path.as_ref() == [filters]);
        let actual = bindings.next().ok_or("required search filters absent")?;
        assert_eq!(
            actual.source,
            BindingSource::Fixed(BindingValue::List(
                vec![
                    filter("AgreementType", "PurchaseAgreement"),
                    filter("PartyType", party),
                ]
                .into_boxed_slice(),
            ))
        );
        assert!(bindings.next().is_none(), "duplicate filters binding");
    }
    Ok(())
}
