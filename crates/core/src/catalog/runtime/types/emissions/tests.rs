use super::*;
use crate::{
    catalog::runtime::{IdentityPlan, ItemPlan, KindClassificationPlan},
    inventory::Kind,
    record::{Exclusions, FieldId},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn descriptor(key: &str, kind: &str) -> TestResult<ResourceEmissionPlan> {
    Ok(ResourceEmissionPlan {
        key: key.into(),
        kind: Kind::parse(kind)?,
        items: ItemPlan::Root,
        identity: IdentityPlan::Key(
            vec![vec![FieldId::new(1)?].into_boxed_slice()].into_boxed_slice(),
        ),
        tags: None,
        exclusions: Exclusions::none(),
        projection: None,
        classification: None,
    })
}

#[test]
fn given_primary_only_when_normalized_then_collection_is_nonempty_and_preserves_primary_descriptor()
-> TestResult {
    let primary = descriptor("primary", "AWS::Route53::DNSSEC")?;
    let emissions = ResourceEmissions::new(primary.clone(), Vec::new())?;
    assert_eq!(emissions.len(), 1);
    assert!(!emissions.is_empty());
    assert_eq!(emissions.primary(), &primary);
    assert_eq!(emissions.iter().collect::<Vec<_>>(), vec![&primary]);
    Ok(())
}

#[test]
fn given_seven_additional_kinds_when_normalized_then_all_eight_descriptors_survive_in_reviewed_order()
-> TestResult {
    let primary = descriptor("primary", "AWS::Test::Primary")?;
    let additional = (0..7)
        .map(|ordinal| {
            descriptor(
                &format!("extra_{ordinal}"),
                &format!("AWS::Test::Extra{ordinal}"),
            )
        })
        .collect::<TestResult<Vec<_>>>()?;
    let expected = std::iter::once(&primary)
        .chain(additional.iter())
        .cloned()
        .collect::<Vec<_>>();
    let emissions = ResourceEmissions::new(primary, additional)?;
    assert_eq!(emissions.len(), 8);
    assert_eq!(emissions.iter().cloned().collect::<Vec<_>>(), expected);
    Ok(())
}

#[test]
fn given_eight_additional_kinds_when_normalized_then_atomic_descriptor_bound_rejects_the_ninth_emitter()
-> TestResult {
    let additional = (0..8)
        .map(|ordinal| {
            descriptor(
                &format!("extra_{ordinal}"),
                &format!("AWS::Test::Extra{ordinal}"),
            )
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert!(matches!(
        ResourceEmissions::new(descriptor("primary", "AWS::Test::Primary")?, additional),
        Err(RuntimeCatalogError::Bounds)
    ));
    Ok(())
}

#[test]
fn given_duplicate_kind_when_descriptors_are_normalized_then_operation_source_cannot_own_the_kind_twice()
-> TestResult {
    let primary = descriptor("primary", "AWS::Route53::DNSSEC")?;
    let additional = descriptor("keys", "AWS::Route53::DNSSEC")?;
    assert!(matches!(
        ResourceEmissions::new(primary, vec![additional]),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_overlapping_classification_kind_when_normalized_then_distinct_base_kinds_do_not_bypass_disjointness()
-> TestResult {
    let primary = descriptor("primary", "AWS::Route53::DNSSEC")?;
    let mut additional = descriptor("keys", "AWS::Route53::KeySigningKey")?;
    additional.classification = Some(KindClassificationPlan {
        path: vec![FieldId::new(2)?].into_boxed_slice(),
        variants: vec![("collision".to_owned(), primary.kind().clone())].into_boxed_slice(),
    });
    assert!(matches!(
        ResourceEmissions::new(primary, vec![additional]),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_sibling_classification_overlap_when_normalized_then_the_second_additional_emitter_is_rejected()
-> TestResult {
    let mut left = descriptor("left", "AWS::Test::Left")?;
    let mut right = descriptor("right", "AWS::Test::Right")?;
    let classification = KindClassificationPlan {
        path: vec![FieldId::new(2)?].into_boxed_slice(),
        variants: vec![("shared".to_owned(), Kind::parse("AWS::Test::Shared")?)].into_boxed_slice(),
    };
    left.classification = Some(classification.clone());
    right.classification = Some(classification);
    assert!(matches!(
        ResourceEmissions::new(
            descriptor("primary", "AWS::Test::Primary")?,
            vec![left, right]
        ),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_invalid_or_duplicate_key_when_normalized_then_descriptor_identity_is_rejected()
-> TestResult {
    ["", "Upper", "has-dash", "contains space", "primary"]
        .into_iter()
        .try_for_each(|key| -> TestResult {
            let primary = descriptor("primary", "AWS::Test::Primary")?;
            let extra = descriptor(key, "AWS::Test::Additional")?;
            assert!(
                matches!(
                    ResourceEmissions::new(primary, vec![extra]),
                    Err(RuntimeCatalogError::Capability)
                ),
                "key={key:?}"
            );
            Ok(())
        })?;
    Ok(())
}

#[test]
fn given_nonprimary_key_when_normalizing_the_discovery_descriptor_then_implicit_role_swapping_is_rejected()
-> TestResult {
    let primary = descriptor("additional", "AWS::Test::Primary")?;
    assert!(matches!(
        ResourceEmissions::new(primary, Vec::new()),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_fixed_emitter_when_observation_claims_another_kind_then_kind_evidence_is_rejected()
-> TestResult {
    use crate::record::{Record, Value};
    let emission = descriptor("primary", "AWS::Test::Primary")?;
    let row = Record::encode(&Value::Map(std::collections::BTreeMap::new()))?;
    assert!(emission.matches_observed_kind(emission.kind(), &row)?);
    assert!(!emission.matches_observed_kind(&Kind::parse("AWS::Test::Foreign")?, &row)?);
    Ok(())
}

#[test]
fn given_classified_emitter_when_record_selects_one_variant_then_declared_siblings_and_implicit_base_are_not_admitted()
-> TestResult {
    use crate::record::{Key, Record, Value};
    let field = FieldId::new(1)?;
    let alpha = Kind::parse("AWS::Test::Alpha")?;
    let beta = Kind::parse("AWS::Test::Beta")?;
    let mut emission = descriptor("primary", "AWS::Test::Base")?;
    emission.classification = Some(KindClassificationPlan {
        path: vec![field].into_boxed_slice(),
        variants: vec![
            ("alpha".to_owned(), alpha.clone()),
            ("beta".to_owned(), beta.clone()),
        ]
        .into_boxed_slice(),
    });
    let row = |label: &str| {
        Record::encode(&Value::Map(std::collections::BTreeMap::from([(
            Key::Field(field),
            Value::Text(label.to_owned()),
        )])))
    };
    assert!(emission.matches_observed_kind(&alpha, &row("alpha")?)?);
    assert!(!emission.matches_observed_kind(&beta, &row("alpha")?)?);
    assert!(!emission.matches_observed_kind(emission.kind(), &row("alpha")?)?);
    assert!(
        matches!(emission.matches_observed_kind(&alpha, &row("unknown")?),
        Err(RuntimeCatalogError::UnclassifiedKind { discriminator }) if discriminator == "unknown")
    );
    Ok(())
}
