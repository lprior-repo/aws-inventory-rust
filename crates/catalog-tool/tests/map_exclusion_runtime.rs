use catalog_tool::{overlay, runtime_catalog::RuntimeCatalogCompileError};
use inventory_core::{
    catalog::runtime::{ModelSourceProvenance, RuntimeCatalog, RuntimeCatalogError},
    record::{Exclusions, FieldId, MapKeyExclusion, Record, Value, ValueRef},
};

#[path = "../../core/tests/support/runtime_provenance.rs"]
mod runtime_provenance;
#[path = "support/map_exclusion.rs"]
mod support;
use support::{
    COUNTER, TestResult, compiled, field, map, masks, name, replaced, rewritten, row, text,
    wire_mask,
};

fn exclusions(bytes: &[u8]) -> TestResult<Exclusions> {
    let runtime = RuntimeCatalog::from_bytes(bytes)?;
    Ok(runtime
        .lookup("things", "ListThings")
        .ok_or("callable plan absent")?
        .hash_exclusions()
        .clone())
}

fn rejects_capability(bytes: &[u8], label: &str) -> TestResult {
    let error = RuntimeCatalog::from_bytes(bytes)
        .err()
        .ok_or("hostile catalog loaded")?;
    assert!(
        matches!(error, RuntimeCatalogError::Capability),
        "{label}: {error}"
    );
    Ok(())
}

#[test]
fn given_reviewed_counters_when_compiled_loaded_and_hashed_then_metadata_remains_and_numeric_masks_coexist()
-> TestResult {
    // Given: only these three SNS-style map keys and the modeled Noise field are reviewed.
    let reviewed = [
        ("Attributes", COUNTER),
        ("Attributes", "SubscriptionsPending"),
        ("Attributes", "SubscriptionsDeleted"),
    ];
    let bytes = compiled(masks(&reviewed)?, true)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("things", "ListThings")
        .ok_or("callable plan absent")?;
    let expected = Exclusions::new([vec![FieldId::new(5)?]])?.with_map_keys(
        reviewed
            .iter()
            .map(|(_, key)| MapKeyExclusion::new(vec![FieldId::new(1)?], *key))
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    assert_eq!(plan.hash_exclusions(), &expected);
    let value = row()?;
    let original = Record::encode(&value)?;
    // When: one reviewed counter or numeric field changes in the retained record.
    for (key, next) in [
        (COUNTER, "101"),
        ("SubscriptionsPending", "202"),
        ("SubscriptionsDeleted", "303"),
    ] {
        let changed = replaced(&value, &[field(1)?, name(key)], text(next))?;
        // Then: hashes ignore that exact key, while retained metadata still exposes its new value.
        assert_eq!(
            original.content_hash(plan.hash_exclusions())?,
            changed.content_hash(plan.hash_exclusions())?
        );
        assert_ne!(
            original.content_hash(&Exclusions::none())?,
            changed.content_hash(&Exclusions::none())?
        );
        let Some(ValueRef::Map(attributes)) = changed.root().get_field(FieldId::new(1)?)? else {
            return Err("retained attributes absent".into());
        };
        assert_eq!(
            attributes
                .get_name(key)?
                .ok_or("retained counter absent")?
                .to_value()?,
            text(next)
        );
        assert_eq!(
            attributes
                .get_name("DisplayName")?
                .ok_or("retained display name absent")?
                .to_value()?,
            text("topic")
        );
    }
    let noise = replaced(&value, &[field(5)?], Value::Int(999))?;
    assert_eq!(
        original.content_hash(plan.hash_exclusions())?,
        noise.content_hash(plan.hash_exclusions())?
    );
    assert_eq!(
        noise
            .root()
            .get_field(FieldId::new(5)?)?
            .ok_or("retained noise absent")?
            .to_value()?,
        Value::Int(999)
    );
    Ok(())
}

#[test]
fn given_one_literal_mask_when_other_keys_or_paths_change_then_hash_remains_sensitive() -> TestResult
{
    // Given: a selected-row map exclusion, not a response-root or wildcard exclusion.
    let bytes = compiled(masks(&[("Attributes", COUNTER)])?, false)?;
    let exclusions = exclusions(&bytes)?;
    let value = row()?;
    let original = Record::encode(&value)?;
    // When: an unrelated key, similarly spelled key, or same key at another path changes.
    for path in [
        vec![field(1)?, name("DisplayName")],
        vec![field(1)?, name("subscriptionsconfirmed")],
        vec![field(1)?, name("SubscriptionsConfirmed.extra")],
        vec![field(1)?, name("SubscriptionsPending")],
        vec![field(1)?, name("65535")],
        vec![field(1)?, field(65_535)?],
        vec![field(6)?, name(COUNTER)],
        vec![field(2)?, field(1)?, name(COUNTER)],
        vec![name("Attributes"), name(COUNTER)],
        vec![name("65535")],
        vec![field(65_535)?],
    ] {
        let changed = replaced(&value, &path, text("changed"))?;
        // Then: no literal or numeric namespace alias broadens the reviewed mask.
        assert_ne!(
            original.content_hash(&exclusions)?,
            changed.content_hash(&exclusions)?,
            "{path:?}"
        );
    }
    Ok(())
}

#[test]
fn given_nested_map_mask_when_compiled_then_only_that_exact_numeric_path_is_omitted() -> TestResult
{
    // Given: the same literal counter at the root map, nested map, and sibling map.
    let bytes = compiled(masks(&[("Envelope.Attributes", COUNTER)])?, false)?;
    let exclusions = exclusions(&bytes)?;
    assert_eq!(
        exclusions.map_keys(),
        &[MapKeyExclusion::new(
            vec![FieldId::new(2)?, FieldId::new(1)?],
            COUNTER
        )?]
    );
    let value = row()?;
    let original = Record::encode(&value)?;
    // When: the nested reviewed key changes.
    let nested = replaced(&value, &[field(2)?, field(1)?, name(COUNTER)], text("100"))?;
    // Then: its hash is stable, but matching keys at other paths are still significant.
    assert_eq!(
        original.content_hash(&exclusions)?,
        nested.content_hash(&exclusions)?
    );
    for path in [
        vec![field(1)?, name(COUNTER)],
        vec![field(6)?, name(COUNTER)],
    ] {
        assert_ne!(
            original.content_hash(&exclusions)?,
            replaced(&value, &path, text("100"))?.content_hash(&exclusions)?
        );
    }
    Ok(())
}

#[test]
fn given_literal_65535_mask_when_hashed_then_numeric_65535_and_root_literal_remain_distinct()
-> TestResult {
    // Given: a literal map key whose spelling resembles a numeric field ID.
    let bytes = compiled(masks(&[("Attributes", "65535")])?, false)?;
    let exclusions = exclusions(&bytes)?;
    let value = row()?;
    let original = Record::encode(&value)?;
    // When: only that literal key changes.
    let literal = replaced(&value, &[field(1)?, name("65535")], text("changed"))?;
    // Then: the literal is omitted without hiding numeric keys or the same name elsewhere.
    assert_eq!(
        original.content_hash(&exclusions)?,
        literal.content_hash(&exclusions)?
    );
    for path in [
        vec![field(1)?, field(65_535)?],
        vec![field(65_535)?],
        vec![name("65535")],
    ] {
        assert_ne!(
            original.content_hash(&exclusions)?,
            replaced(&value, &path, text("changed"))?.content_hash(&exclusions)?
        );
    }
    Ok(())
}

#[test]
fn given_optional_absent_or_reviewed_key_when_loaded_then_only_reviewed_presence_is_hash_neutral()
-> TestResult {
    // Given: identical real compiler fixtures with and without an explicit map-key review.
    let reviewed = compiled(masks(&[("Attributes", COUNTER)])?, false)?;
    let unreviewed = compiled(Vec::new(), false)?;
    let value = row()?;
    let without = replaced(&value, &[field(1)?], Value::Map(Default::default()))?;
    let with = replaced(
        &value,
        &[field(1)?],
        Value::Map([(name(COUNTER), text("1"))].into()),
    )?;
    // When: the sole key appears in an otherwise empty modeled map.
    let mask = exclusions(&reviewed)?;
    // Then: omission includes key presence, while the absent policy retains sensitivity.
    assert_eq!(without.content_hash(&mask)?, with.content_hash(&mask)?);
    assert_eq!(exclusions(&unreviewed)?, Exclusions::none());
    assert_ne!(
        without.content_hash(&exclusions(&unreviewed)?)?,
        with.content_hash(&exclusions(&unreviewed)?)?
    );
    Ok(())
}

#[test]
fn given_maximum_reviewed_masks_and_utf8_key_when_compiled_then_roundtrip_is_deterministic_and_bounded()
-> TestResult {
    // Given: 64 unique masks, including a one-byte key and exactly 4096 UTF-8 bytes.
    let longest = "é".repeat(2048);
    let mut reviewed = masks(&[("Attributes", "x"), ("Attributes", &longest)])?;
    let attributes = overlay::MemberPath::new("Attributes")?;
    reviewed.extend((0..62).map(|index| overlay::MapKeyExclusion {
        path: attributes.clone(),
        key: format!("key-{index}"),
    }));
    // When: the same reviewed model compiles twice and the strict consumer loads it.
    let first = compiled(reviewed.clone(), false)?;
    let second = compiled(reviewed, false)?;
    let runtime = RuntimeCatalog::from_bytes(&first)?;
    let mask = exclusions(&first)?;
    // Then: exact boundary values survive and the receipt honestly identifies assembled input.
    assert_eq!(first, second);
    assert_eq!(mask.map_keys().len(), 64);
    assert_eq!(
        mask.map_keys().iter().map(|mask| mask.key().len()).max(),
        Some(4096)
    );
    assert!(mask.map_keys().iter().any(|mask| mask.key() == longest));
    assert!(matches!(
        runtime.source_provenance().models,
        ModelSourceProvenance::CompiledCatalog { .. }
    ));
    assert_eq!(runtime.source_provenance().cfn_sha256, None);
    let value = map([(
        1,
        Value::Map(
            [
                (name(&longest), text("1")),
                (name("x"), text("1")),
                (name("key-61"), text("1")),
                (name("unreviewed"), text("1")),
            ]
            .into(),
        ),
    )])?;
    let original = Record::encode(&value)?;
    for key in [&longest[..], "x", "key-61"] {
        assert_eq!(
            original.content_hash(&mask)?,
            replaced(&value, &[field(1)?, name(key)], text("2"))?.content_hash(&mask)?
        );
    }
    assert_ne!(
        original.content_hash(&mask)?,
        replaced(&value, &[field(1)?, name("unreviewed")], text("2"))?.content_hash(&mask)?
    );
    Ok(())
}

#[test]
fn given_nonmap_or_unresolved_selected_paths_when_compiling_then_no_executable_mask_is_emitted()
-> TestResult {
    // Given: invalid targets include scalar, structure, list, wrong key, and aggregate values.
    for path in [
        "Missing",
        "Items.Attributes",
        "Scalar",
        "Struct",
        "Lists",
        "IntKeys",
        "Aggregate",
        "ListValues",
        "DocumentValues",
        "Attributes.Missing",
    ] {
        // When: a review requests a map-key exclusion against that target.
        let error = compiled(masks(&[(path, COUNTER)])?, false)
            .err()
            .ok_or("invalid mask compiled")?;
        // Then: the compiler rejects the review rather than emitting a broad/no-op exclusion.
        assert!(
            matches!(error.downcast_ref::<RuntimeCatalogCompileError>(), Some(RuntimeCatalogCompileError::CapabilityContext { service, operation, .. }) if service == "things" && operation == "ListThings"),
            "{path}: {error}"
        );
    }
    Ok(())
}

#[test]
fn given_empty_oversized_or_duplicate_reviews_when_compiling_then_masks_are_rejected() -> TestResult
{
    // Given: invalid literal byte bounds, duplicate masks, or more than 64 unique masks.
    let oversized = "é".repeat(2049);
    let many = (0..65)
        .map(|index| {
            Ok(overlay::MapKeyExclusion {
                path: overlay::MemberPath::new("Attributes")?,
                key: format!("key-{index}"),
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    for reviewed in [
        masks(&[("Attributes", "")])?,
        masks(&[("Attributes", &oversized)])?,
        masks(&[("Attributes", COUNTER), ("Attributes", COUNTER)])?,
        many,
    ] {
        // When: the public compiler builds runtime metadata.
        let error = compiled(reviewed, false)
            .err()
            .ok_or("invalid review compiled")?;
        // Then: no executable catalog is returned.
        assert!(
            matches!(
                error.downcast_ref::<RuntimeCatalogCompileError>(),
                Some(RuntimeCatalogCompileError::CapabilityContext { .. })
            ),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn given_honestly_resealed_valid_mask_when_loaded_then_consumer_hash_behavior_survives()
-> TestResult {
    // Given: real compiled source pins and a valid replacement field 21.
    let base = compiled(masks(&[("Attributes", COUNTER)])?, false)?;
    // When: the public fixture helper re-seals the same metadata using the actual source receipt.
    let bytes = rewritten(&base, Value::List(vec![wire_mask(&[1], COUNTER)?]))?;
    let mask = exclusions(&bytes)?;
    let value = row()?;
    // Then: successful loading and hashing prove malformed tests exercise the intended decoder.
    assert_eq!(mask, exclusions(&base)?);
    assert_eq!(
        Record::encode(&value)?.content_hash(&mask)?,
        replaced(&value, &[field(1)?, name(COUNTER)], text("100"))?.content_hash(&mask)?
    );
    Ok(())
}

#[test]
fn given_resealed_malformed_field21_when_loading_then_strict_consumer_rejects_each_shape()
-> TestResult {
    // Given: an honest compiler-produced receipt, not invented repository/CFN evidence.
    let base = compiled(masks(&[("Attributes", COUNTER)])?, false)?;
    let path = Value::List(vec![Value::Int(1)]);
    let malformed = [
        ("null", Value::Null),
        ("scalar", Value::Int(1)),
        ("map", map([])?),
        ("nonmap mask", Value::List(vec![Value::Null])),
        ("missing fields", Value::List(vec![map([])?])),
        ("missing key", Value::List(vec![map([(1, path.clone())])?])),
        (
            "missing path",
            Value::List(vec![map([(2, text(COUNTER))])?]),
        ),
        (
            "unknown field",
            Value::List(vec![map([
                (1, path.clone()),
                (2, text(COUNTER)),
                (3, Value::Null),
            ])?]),
        ),
        (
            "nonnumeric field",
            Value::List(vec![Value::Map(
                [(name("1"), path.clone()), (field(2)?, text(COUNTER))].into(),
            )]),
        ),
        (
            "nonlist path",
            Value::List(vec![map([(1, Value::Int(1)), (2, text(COUNTER))])?]),
        ),
        ("empty path", Value::List(vec![wire_mask(&[], COUNTER)?])),
        (
            "nonnumeric path",
            Value::List(vec![map([
                (1, Value::List(vec![text("Attributes")])),
                (2, text(COUNTER)),
            ])?]),
        ),
        (
            "negative path",
            Value::List(vec![map([
                (1, Value::List(vec![Value::Int(-1)])),
                (2, text(COUNTER)),
            ])?]),
        ),
        (
            "nontext key",
            Value::List(vec![map([(1, path), (2, Value::Int(1))])?]),
        ),
        ("empty key", Value::List(vec![wire_mask(&[1], "")?])),
        (
            "duplicate masks",
            Value::List(vec![wire_mask(&[1], COUNTER)?, wire_mask(&[1], COUNTER)?]),
        ),
    ];
    for (label, replacement) in malformed {
        // When: the malformed field is re-sealed so checksum rejection cannot mask decoding bugs.
        let bytes = rewritten(&base, replacement)?;
        // Then: malformed capabilities are rejected, never silently treated as absent.
        rejects_capability(&bytes, label)?;
    }
    Ok(())
}

#[test]
fn given_resealed_wrong_model_targets_when_loading_then_numeric_paths_must_resolve_to_scalar_maps()
-> TestResult {
    // Given: numeric IDs resolve against the actual selected item's model.
    let base = compiled(masks(&[("Attributes", COUNTER)])?, false)?;
    for path in [
        vec![999],
        vec![3],
        vec![8],
        vec![9],
        vec![10],
        vec![11],
        vec![12],
        vec![13],
        vec![4, 1],
        vec![1, 1],
    ] {
        // When: a syntactically valid path names no map, an aggregate value, or a wrong key type.
        let bytes = rewritten(&base, Value::List(vec![wire_mask(&path, COUNTER)?]))?;
        // Then: the strict consumer independently rejects the model mismatch.
        rejects_capability(&bytes, &format!("{path:?}"))?;
    }
    Ok(())
}

#[test]
fn given_resealed_oversized_mask_fields_when_loading_then_exact_resource_bounds_are_enforced()
-> TestResult {
    // Given: one-over key length, mask count, or compiled numeric path length.
    let base = compiled(masks(&[("Attributes", COUNTER)])?, false)?;
    let masks = (0..65)
        .map(|index| wire_mask(&[1], &format!("key-{index}")))
        .collect::<TestResult<Vec<_>>>()?;
    let oversized = [
        Value::List(vec![wire_mask(&[1], &"a".repeat(4097))?]),
        Value::List(masks),
        Value::List(vec![wire_mask(&[1; 9], COUNTER)?]),
    ];
    for replacement in oversized {
        // When: genuine source pins and the new reviewed-input checksum are retained.
        let bytes = rewritten(&base, replacement)?;
        let error = RuntimeCatalog::from_bytes(&bytes)
            .err()
            .ok_or("oversized mask loaded")?;
        // Then: rejection is specifically a bound violation, not stale receipt evidence.
        assert!(matches!(error, RuntimeCatalogError::Bounds), "{error}");
    }
    Ok(())
}

#[test]
fn given_same_literal_at_two_reviewed_paths_when_compiled_then_both_masks_apply_without_hiding_a_sibling()
-> TestResult {
    // Given: identical keys are distinct reviews when their numeric map paths differ.
    let bytes = compiled(
        masks(&[("Attributes", COUNTER), ("Envelope.Attributes", COUNTER)])?,
        false,
    )?;
    let mask = exclusions(&bytes)?;
    assert_eq!(mask.map_keys().len(), 2);
    let value = row()?;
    let original = Record::encode(&value)?;
    // When: either explicitly reviewed location changes.
    for path in [
        vec![field(1)?, name(COUNTER)],
        vec![field(2)?, field(1)?, name(COUNTER)],
    ] {
        // Then: both reviews survive sorting and loading, but an unreviewed sibling remains sensitive.
        assert_eq!(
            original.content_hash(&mask)?,
            replaced(&value, &path, text("100"))?.content_hash(&mask)?
        );
    }
    assert_ne!(
        original.content_hash(&mask)?,
        replaced(&value, &[field(6)?, name(COUNTER)], text("100"))?.content_hash(&mask)?
    );
    Ok(())
}
