mod behavior;
mod bound;
mod diagnostics;
use super::*;
use crate::overlay::{
    Identity, ItemSelector, MemberPath, OperationOverlay, OverlayClass, Paginator, Scope,
    ServiceOverlay, TokenPair,
};
use inventory_core::record::{FieldId, ValueRef};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"shapes":{
"n#Svc":{"type":"service","version":"2020-01-01","traits":{"aws.protocols#restXml":{}}},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#readonly":{},"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Input":{"type":"structure","members":{
"StartName":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"name"}},
"StartType":{"target":"n#Kind","traits":{"smithy.api#httpQuery":"type"}},
"StartId":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"identifier"}}}},
"n#Kind":{"type":"enum","members":{"A":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"A"}}}},
"n#Output":{"type":"structure","members":{
"Items":{"target":"n#Rows"},"NextName":{"target":"smithy.api#String"},
"NextType":{"target":"n#Kind"},"NextId":{"target":"smithy.api#String"},"IsTruncated":{"target":"smithy.api#Boolean"}}},
"n#Rows":{"type":"list","member":{"target":"n#Row"}},
"n#Row":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;
type Pair<'a> = (&'a str, &'a str, bool);
const PAIRS: &[Pair<'_>] = &[
    ("StartName", "NextName", false),
    ("StartType", "NextType", false),
    ("StartId", "NextId", true),
];
const REGISTRY: &str =
    "A\nId\nIsTruncated\nItems\nNextId\nNextName\nNextType\nStartId\nStartName\nStartType\n";

fn compile_fixture(model: &str, pairs: &[Pair<'_>]) -> Result<Value, Box<dyn std::error::Error>> {
    compile_fixture_with_registry(model, pairs, REGISTRY)
}

fn compile_fixture_with_registry(
    model: &str,
    pairs: &[Pair<'_>],
    registry: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let service_dir = directory.path().join("models/example/service/2020-01-01");
    std::fs::create_dir_all(&service_dir)?;
    std::fs::write(service_dir.join("example.json"), model)?;
    let registry = crate::smithy::Registry::parse(registry)?;
    let bytes = crate::smithy::build_catalog(directory.path(), &registry, "fixture")?;
    let catalog = inventory_core::catalog::Catalog::borrow(&bytes)?;
    let services = crate::runtime_catalog::ServiceIndex::build(catalog)?;
    let service = catalog.service("example")?.ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    let pairs = pairs
        .iter()
        .map(|(input, output, optional)| {
            Ok(TokenPair {
                input: MemberPath::new(input)?,
                output: MemberPath::new(output)?,
                optional: *optional,
            })
        })
        .collect::<Result<Vec<_>, crate::overlay::NameError>>()?;
    let overlay = ServiceOverlay::new(
        "example",
        vec![(
            "ListThings",
            OperationOverlay {
                class: Some(OverlayClass::Enumerate),
                scope: Some(Scope::Account),
                items: Some(ItemSelector::parse("Items")?),
                identity: Some(Identity::Key(vec![MemberPath::new("Id")?])),
                paginator: Some(Paginator::Tokens {
                    pairs,
                    truncated: Some(MemberPath::new("IsTruncated")?),
                }),
                ..OperationOverlay::default()
            },
        )],
    )?;
    let plan = overlay.operations().values().next().ok_or("review")?;
    Ok(compile(plan, service, operation, 6, &services)?)
}

fn tokens(value: &Value) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let record = inventory_core::record::Record::encode(value)?;
    match record.root().get_field(FieldId::new(2)?)? {
        Some(ValueRef::List(values)) => values
            .iter()?
            .map(|value| value?.to_value())
            .collect::<Result<_, _>>()
            .map_err(Into::into),
        _ => Err("missing compiled tokens".into()),
    }
}

fn compile_last_item_fixture(model: &str) -> Result<Value, Box<dyn std::error::Error>> {
    compile_overlay(
        model,
        &OperationOverlay {
            items: Some(ItemSelector::parse("Items")?),
            paginator: Some(Paginator::LastListItem {
                input: MemberPath::new("StartName")?,
                output: MemberPath::new("NextName")?,
                truncated: MemberPath::new("IsTruncated")?,
            }),
            ..OperationOverlay::default()
        },
    )
}

fn compile_overlay(
    model: &str,
    overlay: &OperationOverlay,
) -> Result<Value, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let service_dir = directory.path().join("models/example/service/2020-01-01");
    std::fs::create_dir_all(&service_dir)?;
    std::fs::write(service_dir.join("example.json"), model)?;
    let registry = crate::smithy::Registry::parse(REGISTRY)?;
    let bytes = crate::smithy::build_catalog(directory.path(), &registry, "fixture")?;
    let catalog = inventory_core::catalog::Catalog::borrow(&bytes)?;
    let services = crate::runtime_catalog::ServiceIndex::build(catalog)?;
    let service = catalog.service("example")?.ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    Ok(compile(overlay, service, operation, 6, &services)?)
}

#[test]
fn last_item_cursor_encodes_scalar_selection_and_truncation() -> TestResult {
    // Given a string-list response cursor and a scalar request cursor.
    let model = MODEL.replace(
        r#""NextName":{"target":"smithy.api#String"}"#,
        r#""NextName":{"target":"n#Names"}"#,
    ).replace(r#""n#Rows":{"type":"list""#, r#""n#Names":{"type":"list","member":{"target":"smithy.api#String"}},"n#Rows":{"type":"list""#);
    // When the reviewed cursor is compiled.
    let value = compile_last_item_fixture(&model)?;
    let compiled = tokens(&value)?;
    assert_eq!(compiled.len(), 1);
    let token = inventory_core::record::Record::encode(compiled.first().ok_or("token")?)?;
    // Then runtime receives last-item selection, not a list-valued request token.
    assert!(matches!(
        token.root().get_field(FieldId::new(7)?)?,
        Some(ValueRef::Int(1))
    ));
    assert!(matches!(
        token.root().get_field(FieldId::new(3)?)?,
        Some(ValueRef::Int(0))
    ));
    assert!(matches!(
        token.root().get_field(FieldId::new(6)?)?,
        Some(ValueRef::Bool(false))
    ));
    let page = inventory_core::record::Record::encode(&value)?;
    let expected = crate::smithy::Registry::parse(REGISTRY)?
        .id("IsTruncated")
        .ok_or("flag id")?;
    let Some(ValueRef::List(flag)) = page.root().get_field(FieldId::new(5)?)? else {
        return Err("missing truncation flag".into());
    };
    assert_eq!(
        flag.iter()?
            .map(|value| value?.to_value())
            .collect::<Result<Vec<_>, _>>()?,
        vec![Value::Int(i64::from(expected.value()))]
    );
    Ok(())
}

#[test]
fn last_item_cursor_rejects_non_list_response() -> TestResult {
    // Given an ordinary scalar output instead of a last-item list cursor.
    // When compiled, then the capability error is returned rather than list semantics being lost.
    let error = compile_last_item_fixture(MODEL)
        .err()
        .ok_or("accepted scalar cursor")?;
    let compile_error = error
        .downcast_ref::<RuntimeCatalogCompileError>()
        .ok_or("wrong error type")?;
    assert!(matches!(compile_error,
        RuntimeCatalogCompileError::Path { path } if path == "NextName"));
    Ok(())
}

#[test]
fn last_item_cursor_rejects_nontext_elements_required_input_and_nonboolean_flag() -> TestResult {
    // Given a valid list cursor, change one modeled invariant at a time.
    let model = MODEL.replace(
        r#""NextName":{"target":"smithy.api#String"}"#,
        r#""NextName":{"target":"n#Names"}"#,
    ).replace(r#""n#Rows":{"type":"list""#,
        r#""n#Names":{"type":"list","member":{"target":"smithy.api#String"}},"n#Rows":{"type":"list""#);
    let cases = [
        (
            "nontext elements",
            model.replace(
                r#""n#Names":{"type":"list","member":{"target":"smithy.api#String"}}"#,
                r#""n#Names":{"type":"list","member":{"target":"smithy.api#Boolean"}}"#,
            ),
            "NextName",
        ),
        (
            "required input",
            model.replace(
                r#""smithy.api#httpQuery":"name""#,
                r#""smithy.api#httpQuery":"name","smithy.api#required":{}"#,
            ),
            "StartName",
        ),
        (
            "nonboolean flag",
            model.replace(
                r#""IsTruncated":{"target":"smithy.api#Boolean"}"#,
                r#""IsTruncated":{"target":"smithy.api#String"}"#,
            ),
            "IsTruncated",
        ),
    ];
    for (name, model, expected_path) in cases {
        // When compiled, then unsafe token/truncation contracts fail explicitly.
        let error = compile_last_item_fixture(&model)
            .err()
            .ok_or_else(|| format!("accepted invalid cursor for {name}"))?;
        let compile_error = error
            .downcast_ref::<RuntimeCatalogCompileError>()
            .ok_or_else(|| format!("wrong error type for {name}"))?;
        assert!(
            matches!(compile_error,
            RuntimeCatalogCompileError::Path { path } if path == expected_path),
            "unexpected error for {name}"
        );
    }
    Ok(())
}

#[test]
fn given_endpoint_exception_when_encoded_then_fields_are_present() -> TestResult {
    use inventory_core::record::{Key, Value};
    let exception = crate::endpoints::EndpointException {
        region: "eu-west-1".to_owned(),
        url: "https://example.eu-west-1.amazonaws.com".to_owned(),
        signing_name: "example".to_owned(),
        signing_region: "eu-west-1".to_owned(),
    };
    let value = crate::runtime_catalog::plans::encode_endpoint_exception(&exception)?;
    let Value::Map(fields) = value else {
        return Err("expected endpoint map".into());
    };
    assert_eq!(
        fields.get(&Key::Field(FieldId::new(1)?)),
        Some(&Value::Text("eu-west-1".to_owned()))
    );
    assert_eq!(
        fields.get(&Key::Field(FieldId::new(2)?)),
        Some(&Value::Text(exception.url))
    );
    assert_eq!(
        fields.get(&Key::Field(FieldId::new(3)?)),
        Some(&Value::Text(exception.signing_name))
    );
    assert_eq!(
        fields.get(&Key::Field(FieldId::new(4)?)),
        Some(&Value::Text(exception.signing_region))
    );
    Ok(())
}
