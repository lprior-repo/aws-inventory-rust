use catalog_tool::smithy::{self, CompileError, Registry};
use inventory_core::{catalog::Catalog, record::ValueRef};
use serde_json::{Value, json};
use std::fs;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn shape(values: Vec<Value>, kind: &str) -> Value {
    if kind == "legacy" {
        return json!({"type":"string","traits":{"smithy.api#enum":values.into_iter().map(|value| json!({"value":value})).collect::<Vec<_>>()}});
    }
    let members = values.into_iter().enumerate().map(|(index, value)| {
        (format!("Value{index}"), json!({"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":value}}))
    }).collect::<serde_json::Map<_, _>>();
    json!({"type":kind,"members":members})
}
fn build(choice: Value) -> TestResult<Result<Vec<u8>, smithy::Error>> {
    let source = tempfile::tempdir()?;
    let model = source.path().join("models/metadata/service/2020-01-01");
    fs::create_dir_all(&model)?;
    let document = json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","version":"1","traits":{"aws.protocols#restJson1":{}}},
        "n#Choice":choice
    }});
    fs::write(model.join("model.json"), serde_json::to_vec(&document)?)?;
    Ok(smithy::build_catalog(source.path(), &Registry::parse("Name\n")?, "enum-metadata-fixture"))
}
fn accepted(choice: Value, value: ValueRef<'_>, member: bool) -> TestResult {
    let bytes = build(choice)??;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("metadata")?.ok_or("service absent")?;
    let shapes = service.shapes()?.collect::<Result<Vec<_>, _>>()?;
    let choice = shapes.iter().find(|shape| shape.name().is_ok_and(|name| name == "Choice")).ok_or("choice absent")?;
    assert_eq!(choice.accepts_enum_value(value)?, member);
    Ok(())
}
fn rejected(choice: Value) -> TestResult {
    let Err(smithy::Error::Compile(CompileError::Service { service, source })) = build(choice)? else {
        return Err("expected contextual compiler service error".into());
    };
    assert_eq!(service, "metadata");
    let CompileError::Shape { shape, source } = *source else { return Err("expected contextual shape error".into()); };
    assert_eq!(shape, "n#Choice");
    let CompileError::Trait(value) = *source else { return Err("expected exact enum trait error".into()); };
    assert_eq!(value, "smithy.api#enumValue or smithy.api#enum");
    Ok(())
}

#[test]
fn given_enum_metadata_count_when_below_at_or_above_limit_then_public_compiler_preserves_or_rejects_exactly() -> TestResult {
    for kind in ["enum", "legacy"] {
        for count in [65_535, 65_536] {
            accepted(shape(vec![json!("wire"); count], kind), ValueRef::Text("wire"), true)?;
        }
        rejected(shape(vec![json!("wire"); 65_537], kind))?;
    }
    Ok(())
}

#[test]
fn given_integer_enum_metadata_count_when_at_or_above_limit_then_public_compiler_preserves_or_rejects_exactly() -> TestResult {
    accepted(shape(vec![json!(7); 65_536], "intEnum"), ValueRef::Int(7), true)?;
    rejected(shape(vec![json!(7); 65_537], "intEnum"))
}

#[test]
fn given_enum_metadata_text_when_utf8_bytes_cross_limit_then_public_compiler_enforces_exact_bound() -> TestResult {
    for kind in ["enum", "legacy"] {
        for value in [format!("{}a", "é".repeat(8_191)), "é".repeat(8_192)] {
            accepted(shape(vec![json!(value)], kind), ValueRef::Text(&value), true)?;
        }
        rejected(shape(vec![json!(format!("{}a", "é".repeat(8_192)))], kind))?;
    }
    Ok(())
}

#[test]
fn given_enum_metadata_table_when_total_text_crosses_limit_then_public_compiler_enforces_exact_bound() -> TestResult {
    let value = "x".repeat(16_384);
    for kind in ["enum", "legacy"] {
        let mut below = vec![json!(value); 63];
        below.push(json!("x".repeat(16_383)));
        accepted(shape(below, kind), ValueRef::Text(&value), true)?;
        let mut at = vec![json!(value); 64];
        accepted(shape(at.clone(), kind), ValueRef::Text(&value), true)?;
        at.push(json!("x"));
        rejected(shape(at, kind))?;
    }
    Ok(())
}

#[test]
fn given_empty_duplicate_and_wildcard_enum_domains_when_compiled_then_membership_is_literal_set_semantics() -> TestResult {
    for kind in ["enum", "legacy"] {
        accepted(shape(vec![], kind), ValueRef::Text("anything"), false)?;
        accepted(shape(vec![json!("*"), json!("*")], kind), ValueRef::Text("*"), true)?;
        accepted(shape(vec![json!("*"), json!("*")], kind), ValueRef::Text("anything"), false)?;
    }
    Ok(())
}

#[test]
fn given_ordinary_string_without_enum_trait_when_compiled_then_explicit_unconstrained_domain_accepts_text() -> TestResult {
    accepted(json!({"type":"string"}), ValueRef::Text("arbitrary ordinary text"), true)
}
