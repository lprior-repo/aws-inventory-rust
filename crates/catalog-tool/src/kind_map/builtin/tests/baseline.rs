use super::*;
#[test]
fn embedded_baseline_preserves_all_333_exact_operation_mappings() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.operations().len(), 333);
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
        "57bff7790dbbf197c41bd3225369ae32aa605116f9d656e602347f44614d967f"
    );
    Ok(())
}

#[test]
fn embedded_baseline_preserves_182_canonical_cfn_kinds() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.entries().len(), 182);
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
        "c6fe97c29d1b9fa212d24dae314c9d3c909c3170958ebb80d8fa0c9c8bd0bb6d"
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
fn embedded_baseline_preserves_all_42_exact_scalar_relationships() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.relations().len(), 50);
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
        "4fdae8425ac3fd43b1c542574bbcd25b0048ca0684e0abdc30b7931e49802082"
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
