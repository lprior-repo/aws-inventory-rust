use super::*;
#[test]
fn embedded_baseline_preserves_all_339_exact_operation_mappings() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.operations().len(), 339);
    let actual = digest(
        map.operations()
            .iter()
            .filter(|row| {
                !NEW_SAFE_METADATA_OPERATIONS
                    .iter()
                    .any(|(service, operation, _)| {
                        row.service == *service && row.operation == *operation
                    })
            })
            .map(|row| format!("{}\t{}\t{}\n", row.service, row.operation, row.kind)),
    );
    assert_eq!(
        actual,
        "4bf986df01d2401237d973dfc88ac99dc2efce3516c66132d46bcb313f91d5ee"
    );
    Ok(())
}

#[test]
fn embedded_baseline_preserves_186_canonical_cfn_kinds() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.entries().len(), 186);
    assert!(map.entries().iter().all(
        |entry| matches!(&entry.target, KindTarget::Mapped { kind, .. } if kind == &entry.cfn_type)
    ));
    let actual = digest(
        map.entries()
            .iter()
            .filter(|row| !CENTRAL_ADDED_CFN_KINDS.contains(&row.cfn_type.as_str()))
            .filter_map(|row| match &row.target {
                KindTarget::Mapped { kind, .. } => {
                    Some(format!("{}\tmap\t{}\t-\n", row.cfn_type, kind))
                }
                KindTarget::Unmapped { .. } => None,
            }),
    );
    assert_eq!(
        actual,
        "a38196786cdda746e250310a830a11040b8d81f48c75c1c8cfb41bba62eb2545"
    );
    Ok(())
}

#[test]
fn embedded_baseline_preserves_all_120_api_only_reasons() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.api_only().len(), 120);
    let actual = digest(
        map.api_only()
            .iter()
            .filter(|row| !CENTRAL_ADDED_API_KINDS.contains(&row.kind.as_str()))
            .map(|row| format!("{}\tkind\t{}\n", row.kind, row.reason)),
    );
    assert_eq!(
        actual,
        "ca17681b5cab4b5ebfa4c8eb05fa830631d062857f95408d1bd70d2ad98d76ea"
    );
    Ok(())
}

#[test]
fn embedded_baseline_preserves_all_57_exact_scalar_relationships() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.relations().len(), 57);
    let actual = digest(
        map.relations()
            .iter()
            .filter(|row| {
                matches!(row.source, ReviewedRelationSource::SelectedOutput(_))
                    && ![
                        "AWS::ApiOnly::IAMServiceSpecificCredential",
                        "AWS::Route53::KeySigningKey",
                    ]
                    .contains(&row.source_kind.as_str())
            })
            .map(scalar_relation_row)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(),
    );
    assert_eq!(
        actual,
        "29b39b7fefbb57a328967340feda721ec7835036d09260199db7c3f7d3652545"
    );
    Ok(())
}

#[test]
fn embedded_registry_accounts_for_every_pinned_cfn_type_without_duplicate_canonical_nodes()
-> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    assert_eq!(map.entries().len(), 1_881);
    assert!(
        map.entries()
            .windows(2)
            .all(|pair| matches!(pair, [left, right] if left.cfn_type < right.cfn_type))
    );
    assert!(map.operations().windows(2).all(|pair| matches!(pair, [left, right] if (&left.service, &left.operation) < (&right.service, &right.operation))));
    Ok(())
}
