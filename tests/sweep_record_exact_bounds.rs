#![allow(dead_code)]
use inventory_core::{
    catalog::runtime::{ItemPlan, RuntimeCatalog},
    record::{FieldId, Float, Key, Record, Time, Value, ValueRef, scalar_item, scalar_record},
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Each envelope is independently specified CBOR: a one-entry map with the text
// key "$". Expected bytes never come from the scalar encoder under test.
fn scalar_cases() -> TestResult<Vec<(ValueRef<'static>, Vec<u8>)>> {
    Ok(vec![
        (ValueRef::Null, vec![0xa1, 0x61, 0x24, 0xf6]),
        (ValueRef::Bool(false), vec![0xa1, 0x61, 0x24, 0xf4]),
        (ValueRef::Bool(true), vec![0xa1, 0x61, 0x24, 0xf5]),
        (ValueRef::Int(i64::MIN), vec![0xa1, 0x61, 0x24, 0x3b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        (ValueRef::Int(i64::MAX), vec![0xa1, 0x61, 0x24, 0x1b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        (ValueRef::Int(0), vec![0xa1, 0x61, 0x24, 0x00]),
        (ValueRef::Text(""), vec![0xa1, 0x61, 0x24, 0x60]),
        (ValueRef::Text("é"), vec![0xa1, 0x61, 0x24, 0x62, 0xc3, 0xa9]),
        (ValueRef::Bytes(&[]), vec![0xa1, 0x61, 0x24, 0x40]),
        (ValueRef::Bytes(&[0, 255]), vec![0xa1, 0x61, 0x24, 0x42, 0, 255]),
        (ValueRef::Float(Float::new(0.5)), vec![0xa1, 0x61, 0x24, 0xfb, 0x3f, 0xe0, 0, 0, 0, 0, 0, 0]),
        (ValueRef::Time(Time::new(-1, 1)?), vec![0xa1, 0x61, 0x24, 0xc1, 0x82, 0x20, 0x01]),
    ])
}

#[test]
fn scalar_decoder_preserves_each_native_scalar_in_an_independently_encoded_envelope() -> TestResult {
    // Given independently encoded scalar envelopes, including both polarities,
    // signed extrema, null and scalar kinds that are not AWS identity strings.
    for (expected, bytes) in scalar_cases()? {
        let record = Record::from_bytes(bytes)?;
        // When the public selected-item decoder reads the envelope.
        let actual = scalar_item(record.root())?.map(|value| value.to_value()).transpose()?;
        // Then native type and value are exact, without paired-encoder oracles.
        assert_eq!(actual, Some(expected.to_value()?));
    }
    Ok(())
}

#[test]
fn scalar_encoder_emits_exactly_one_dollar_entry_with_independently_expected_bytes() -> TestResult {
    for (value, expected) in scalar_cases()? {
        // Given a native scalar; when wrapped by the public scalar encoder.
        let record = scalar_record(value)?;
        // Then the bytes and entry count match the independent envelope oracle.
        assert_eq!(record.as_bytes(), expected.as_slice());
        assert_eq!(record.root().iter()?.collect::<Result<Vec<_>, _>>()?.len(), 1);
    }
    Ok(())
}

#[test]
fn scalar_decoder_refuses_extra_entries_containers_and_non_dollar_structures() -> TestResult {
    // All are valid records, but none is a selected-scalar envelope. In
    // particular "$" comes first in the two-entry named map, exercising the
    // second-entry guard rather than rejection of the first key alone.
    let cases: &[(&str, &[u8])] = &[
        ("extra named entry", &[0xa2, 0x61, 0x24, 0x61, b'x', 0x61, b'z', 0x01]),
        ("extra ordinary field", &[0xa2, 0x01, 0x01, 0x61, 0x24, 0x61, b'x']),
        ("empty map payload", &[0xa1, 0x61, 0x24, 0xa0]),
        ("populated map payload", &[0xa1, 0x61, 0x24, 0xa1, 0x01, 0x01]),
        ("empty list payload", &[0xa1, 0x61, 0x24, 0x80]),
        ("populated list payload", &[0xa1, 0x61, 0x24, 0x81, 0xf6]),
        ("different named key", &[0xa1, 0x61, b'x', 0x61, b'a']),
        ("empty structure", &[0xa0]),
        ("ordinary field structure", &[0xa1, 0x01, 0x61, b'a']),
    ];
    for (name, bytes) in cases {
        // Given a hostile shape encoded without scalar_record.
        let record = Record::from_bytes(bytes.to_vec())?;
        // When decoded, then it is exactly absent, not a scalar or parse error.
        assert_eq!(scalar_item(record.root())?.map(|value| value.to_value()).transpose()?, None, "{name}");
    }
    Ok(())
}

// These helpers are also attached to the private production record module by
// two #[test] wrappers. The standalone integration target above needs no seam;
// only those wrappers can call the real private extraction boundary below.
pub(crate) fn accepts_exact_page<E>(
    extract: impl FnOnce(&Record, &ItemPlan) -> Result<Vec<Record>, E>,
) -> TestResult
where
    E: std::error::Error + 'static,
{
    // Given a real compiled scalar enumeration and precisely 100000 items.
    let bytes = compiled_scalar_catalog()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("bounds", "ListThings").ok_or("compiled plan missing")?;
    let response = response(100_000)?;
    // When the production extraction consumer uses the loaded selector.
    let rows = extract(&response, plan.items())?;
    // Then every accepted row is preserved in order with its full native value.
    assert_eq!(rows.len(), 100_000);
    for (index, row) in rows.iter().enumerate() {
        let actual = scalar_item(row.root())?.map(|value| value.to_value()).transpose()?;
        assert_eq!(actual, Some(Value::Int(i64::try_from(index)?)), "row {index}");
        assert_eq!(row.root().iter()?.collect::<Result<Vec<_>, _>>()?.len(), 1);
    }
    Ok(())
}

pub(crate) fn rejects_oversized_page<E>(
    extract: impl FnOnce(&Record, &ItemPlan) -> Result<Vec<Record>, E>,
) -> TestResult
where
    E: std::error::Error + 'static,
{
    // Given the same compiled plan, with precisely one additional valid item.
    let bytes = compiled_scalar_catalog()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("bounds", "ListThings").ok_or("compiled plan missing")?;
    let response = response(100_001)?;
    // When extracting the page, then no successful partial Vec can escape.
    match extract(&response, plan.items()) {
        Err(error) => {
            assert_eq!(format!("{error:?}"), "Operation(\"page exceeds the atomic observation bound\")");
            assert_eq!(error.to_string(), "sweep operation failed: page exceeds the atomic observation bound");
        }
        Ok(rows) => return Err(format!("oversized page returned {} partial or unbounded rows", rows.len()).into()),
    }
    Ok(())
}

fn response(count: i64) -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        Value::List((0..count).map(Value::Int).collect()),
    )])))?)
}

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Bounds","endpointPrefix":"bounds","arnNamespace":"bounds"},"aws.auth#sigv4":{"name":"bounds"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"smithy.api#Long"}}
}}"#;

fn compiled_scalar_catalog() -> TestResult<Vec<u8>> {
    use catalog_tool::{classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy};
    use serde_json::json;
    use std::{fs, path::Path};

    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let models = source.join("models/bounds/service/2026-01-01");
    fs::create_dir_all(&models)?;
    fs::write(models.join("model.json"), MODEL)?;
    let mut operation = overlay::OperationOverlay::default();
    operation.class = Some(overlay::OverlayClass::Enumerate);
    operation.scope = Some(overlay::Scope::Region);
    operation.items = Some(overlay::ItemSelector::Member(overlay::MemberPath::new("Items")?));
    operation.paginator = Some(overlay::Paginator::None);
    operation.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::selected("$")?]));
    let policies = [overlay::ServiceOverlay::new("bounds", vec![("ListThings", operation)])?];
    let checked = overlay::check_policies(&source, &policies, &DenyList::parse("")?)?;
    let fields = smithy::Registry::parse("Items\n")?;
    let base = smithy::build_catalog(&source, &fields, "exact-bound-fixture")?;
    let kinds = kind_map::MappingSource {
        version: 1, reviewed: "2026-11-15", reviewer: "Exact-bound fixture",
        default_unmapped_reason: None, cfn: &[], relations: &[], parent_relations: &[], compound_relations: &[], additional_emissions: &[], relation_targets: &[], bootstraps: &[],
        api_only: &[kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::ExactBound", reason: "Exact-bound scalar fixture",
        }],
        operations: &[kind_map::OperationMapping {
            service: "bounds", operation: "ListThings", kind: "AWS::ApiOnly::ExactBound",
        }],
    }.compile(&names::Exceptions::none())?;
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"bounds","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://bounds.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("exact-bound-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://bounds.test","properties":properties}}}]))?;
    let endpoint = rules.run_cases(&cases)?.verified_endpoint(Some("bounds"))
        .map_err(|_| "fixture endpoint evidence failed")?;
    let endpoints = BTreeMap::from([("bounds".to_owned(), endpoint)]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?)
}
