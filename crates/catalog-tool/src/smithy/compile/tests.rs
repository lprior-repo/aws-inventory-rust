use super::*;
use crate::smithy::{model::ModelFile, stats};
use inventory_core::catalog::{Catalog, Target};
use std::collections::BTreeMap;
type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"shapes":{
 "n#Svc":{"type":"service","version":"2020-01-01","traits":{"aws.protocols#restJson1":{}}},
 "n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Things"},
   "traits":{"smithy.api#readonly":{},"smithy.api#paginated":{}}},
 "n#Input":{"type":"structure","members":{"Zulu":{"target":"smithy.api#String"},"Alpha":{"target":"n#Choice"}}},
 "n#Choice":{"type":"union","members":{"Alpha":{"target":"smithy.api#Boolean","traits":{"smithy.api#required":{}}}}},
 "n#Things":{"type":"list","member":{"target":"n#Input"}},
 "n#Lookup":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Things"}},
 "n#Thing":{"type":"resource","traits":{"aws.api#arn":{"template":"thing/{id}"}}},
 "n#Text":{"type":"string"}
}}"#;
fn record(text: &str, registry: &str) -> Result<Record, Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(text)?;
    stats::service(&model, Path::new("fixture"))?;
    Ok(encode::service(
        &model,
        &Registry::parse(registry)?,
        "example",
    )?)
}
fn shape_failure(
    text: &str,
    registry: &str,
) -> Result<(String, CompileError), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(text)?;
    let Err(CompileError::Shape { shape, source }) =
        encode::service(&model, &Registry::parse(registry)?, "example")
    else {
        return Err("expected a contextual shape error".into());
    };
    Ok((shape, *source))
}
fn service_failure(
    source: &Path,
    expected: &str,
) -> Result<CompileError, Box<dyn std::error::Error>> {
    let result = build_catalog(source, &Registry::parse("Zulu\nAlpha\n")?, "fixture");
    let Err(Error::Compile(CompileError::Service { service, source })) = result else {
        return Err("expected a contextual service error".into());
    };
    assert_eq!(service, expected);
    Ok(*source)
}
fn root(service: Record) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let root = encode::map([
        (CatalogKey::Format, Value::Int(2)),
        (CatalogKey::ModelsCommit, Value::Text("fixture".into())),
        (CatalogKey::FieldNames, Value::Bytes(b"ZuluAlpha".to_vec())),
        (
            CatalogKey::FieldNameOffsets,
            Value::Bytes(vec![0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 9]),
        ),
        (
            CatalogKey::Services,
            Value::List(vec![Value::Bytes(service.into_bytes())]),
        ),
    ])?;
    Ok(Record::encode(&root)?.into_bytes())
}
#[test]
fn compiler_preserves_registry_member_order_and_targets() -> TestResult {
    let bytes = root(record(MODEL, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let input = service.shape(1)?.ok_or("input")?;
    let members = input.members()?.collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<_> = members.iter().map(|m| m.id.value()).collect();
    assert_eq!(ids, vec![1, 2]);
    assert_eq!(
        members.first().ok_or("member")?.target,
        Target::Prelude("string")
    );
    assert_eq!(members.get(1).ok_or("member")?.target, Target::Shape(0));
    Ok(())
}
#[test]
fn compiler_preserves_container_relationships() -> TestResult {
    let bytes = root(record(MODEL, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    assert_eq!(service.shape_count(), 8);
    let list = service.shape(7)?.ok_or("list")?;
    assert_eq!(list.element()?, Some(Target::Shape(1)));
    let map = service.shape(3)?.ok_or("map")?;
    assert_eq!(map.map_key()?, Some(Target::Prelude("string")));
    assert_eq!(map.map_value()?, Some(Target::Shape(7)));
    Ok(())
}
#[test]
fn compiler_preserves_union_requiredness_and_resource_templates() -> TestResult {
    let bytes = root(record(MODEL, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let choice = service.shape(0)?.ok_or("choice")?;
    assert_eq!(choice.kind()?, "union");
    assert!(choice.members()?.next().ok_or("member")??.required);
    assert_eq!(
        service.shape(6)?.ok_or("resource")?.arn()?,
        Some("thing/{id}")
    );
    Ok(())
}
#[test]
fn compilation_is_identical_despite_json_member_and_shape_order() -> TestResult {
    let value: serde_json::Value = serde_json::from_str(MODEL)?;
    let compact = serde_json::to_string(&value)?;
    assert_eq!(
        record(MODEL, "Zulu\nAlpha\n")?,
        record(&compact, "Zulu\nAlpha\n")?
    );
    Ok(())
}
#[test]
fn missing_registry_name_is_actionable_and_never_allocates_an_id() -> TestResult {
    let (shape, cause) = shape_failure(MODEL, "Zulu\n")?;
    assert_eq!(shape, "n#Choice");
    let CompileError::MissingMember(name) = cause else {
        return Err("expected missing member error".into());
    };
    assert_eq!(name, "Alpha");
    assert!(
        CompileError::MissingMember(name)
            .to_string()
            .contains("models fields --write")
    );
    Ok(())
}
#[test]
fn unknown_target_retains_its_owning_shape_and_typed_cause() -> TestResult {
    let bad = MODEL.replace("smithy.api#String", "n#Missing");
    let (shape, cause) = shape_failure(&bad, "Zulu\nAlpha\n")?;
    assert_eq!(shape, "n#Input");
    assert!(matches!(cause, CompileError::Target(name) if name == "n#Missing"));
    Ok(())
}
#[test]
fn unknown_kind_retains_its_owning_shape_and_typed_cause() -> TestResult {
    let bad = MODEL.replace("\"type\":\"string\"", "\"type\":\"future\"");
    let (shape, cause) = shape_failure(&bad, "Zulu\nAlpha\n")?;
    assert_eq!(shape, "n#Text");
    assert!(matches!(cause, CompileError::Kind));
    Ok(())
}
#[test]
fn compiler_report_matches_independent_model_reader() -> TestResult {
    let source = tempfile::tempdir()?;
    std::fs::create_dir_all(source.path().join("models/example"))?;
    std::fs::write(source.path().join("models/example/model.json"), MODEL)?;
    let registry = Registry::parse("Zulu\nAlpha\n")?;
    let bytes = build_catalog(source.path(), &registry, "fixture")?;
    assert_eq!(
        Catalog::borrow(&bytes)?.counts()?.render()?,
        crate::smithy::build_report(source.path())?
    );
    assert_eq!(bytes, build_catalog(source.path(), &registry, "fixture")?);
    Ok(())
}
#[test]
fn primitive_and_big_number_preludes_have_distinct_kind_codes() -> TestResult {
    let indexes = std::collections::BTreeMap::new();
    assert_eq!(
        encode::reference("smithy.api#PrimitiveBoolean", &indexes)?,
        Value::Int(-2)
    );
    assert_eq!(
        encode::reference("smithy.api#BigInteger", &indexes)?,
        Value::Int(-10)
    );
    assert_eq!(
        encode::reference("smithy.api#BigDecimal", &indexes)?,
        Value::Int(-11)
    );
    assert_eq!(
        encode::reference("smithy.api#Unit", &indexes)?,
        Value::Int(-14)
    );
    Ok(())
}

#[test]
fn prelude_unit_uses_builtin_identity_before_local_shape_index() -> TestResult {
    let indexes = BTreeMap::from([("smithy.api#Unit", 0)]);
    assert_eq!(
        encode::reference("smithy.api#Unit", &indexes)?,
        Value::Int(-14)
    );
    Ok(())
}

#[test]
fn unknown_smithy_api_shape_uses_only_an_explicit_local_index() -> TestResult {
    let indexes = BTreeMap::from([("smithy.api#PrivateShape", 4)]);
    assert_eq!(
        encode::reference("smithy.api#PrivateShape", &indexes)?,
        Value::Int(4)
    );
    assert!(matches!(
        encode::reference("smithy.api#UnknownShape", &indexes),
        Err(CompileError::Target(id)) if id == "smithy.api#UnknownShape"
    ));
    Ok(())
}
#[test]
fn shared_reader_compiles_inherited_blob_default_and_client_optional() -> TestResult {
    let source = tempfile::tempdir()?;
    let directory = source.path().join("models/example");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(
        directory.join("model.json"),
        r#"{"shapes":{
        "n#Svc":{"type":"service","version":"2024-01-01","traits":{"aws.protocols#restJson1":{}}},
        "n#Base":{"type":"structure","traits":{"smithy.api#mixin":{}},"members":{"value":{"target":"smithy.api#Blob","traits":{"smithy.api#default":"AQID","smithy.api#clientOptional":{}}}}},
        "n#Input":{"type":"structure","mixins":[{"target":"n#Base"}]},
        "n#Call":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"smithy.api#Unit"}}
    }}"#,
    )?;
    let bytes = build_catalog(source.path(), &Registry::parse("value\n")?, "fixture")?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let member = service
        .shape(2)?
        .ok_or("input")?
        .members()?
        .next()
        .ok_or("member")??;
    assert_eq!(member.target, Target::Prelude("blob"));
    assert!(
        matches!(member.default_value()?, Some(inventory_core::record::ValueRef::Bytes(bytes)) if bytes == &[1, 2, 3])
    );
    assert!(member.client_optional()?);
    Ok(())
}

mod contexts;
mod identities;
mod values;
