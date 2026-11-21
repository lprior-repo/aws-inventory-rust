mod behavior;
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
    assert_eq!(
        error
            .downcast_ref::<RuntimeCatalogCompileError>()
            .map(ToString::to_string),
        Some(RuntimeCatalogCompileError::Capability.to_string())
    );
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
        model.replace(
            r#""n#Names":{"type":"list","member":{"target":"smithy.api#String"}}"#,
            r#""n#Names":{"type":"list","member":{"target":"smithy.api#Boolean"}}"#,
        ),
        model.replace(
            r#""smithy.api#httpQuery":"name""#,
            r#""smithy.api#httpQuery":"name","smithy.api#required":{}"#,
        ),
        model.replace(
            r#""IsTruncated":{"target":"smithy.api#Boolean"}"#,
            r#""IsTruncated":{"target":"smithy.api#String"}"#,
        ),
    ];
    for model in cases {
        // When compiled, then unsafe token/truncation contracts fail explicitly.
        let error = compile_last_item_fixture(&model)
            .err()
            .ok_or("accepted invalid cursor")?;
        assert_eq!(
            error
                .downcast_ref::<RuntimeCatalogCompileError>()
                .map(ToString::to_string),
            Some(RuntimeCatalogCompileError::Capability.to_string())
        );
    }
    Ok(())
}

const BOUND_REGISTRY: &str = "Cursor\nId\nItems\nLimit\nNextCursor\n";
const BOUND_KINDS: crate::kind_map::MappingSource<'static> = crate::kind_map::MappingSource {
    version: 1,
    reviewed: "2026-11-15",
    reviewer: "Paginator protocol binding fixture",
    default_unmapped_reason: None,
    cfn: &[],
    relations: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
    relation_targets: &[],
    bootstraps: &[],
    api_only: &[crate::kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::Thing",
        reason: "Paginator protocol binding fixture",
    }],
    operations: &[crate::kind_map::OperationMapping {
        service: "example",
        operation: "ListThings",
        kind: "AWS::ApiOnly::Thing",
    }],
};

fn bound_paginator_model(protocol: &str, binding: serde_json::Value) -> serde_json::Value {
    let limit_binding = match binding.get("smithy.api#httpQuery") {
        Some(_) => serde_json::json!({"smithy.api#httpQuery":"limit"}),
        None => binding.clone(),
    };
    serde_json::json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","version":"2020-01-01",
            "operations":[{"target":"n#ListThings"}],"traits":{
                "aws.api#service":{"sdkId":"Example","endpointPrefix":"example","arnNamespace":"example"},
                "aws.auth#sigv4":{"name":"example"},(protocol):{}}},
        "n#ListThings":{"type":"operation","input":{"target":"n#Input"},
            "output":{"target":"n#Output"},"traits":{"smithy.api#readonly":{},
                "smithy.api#http":{"method":"POST","uri":"/things"},
                "smithy.api#paginated":{"inputToken":"Cursor","outputToken":"NextCursor",
                    "items":"Items","pageSize":"Limit"}}},
        "n#Input":{"type":"structure","members":{
            "Cursor":{"target":"smithy.api#String","traits":binding},
            "Limit":{"target":"n#Limit","traits":limit_binding}}},
        "n#Limit":{"type":"integer","traits":{"smithy.api#range":{"min":1,"max":100}}},
        "n#Output":{"type":"structure","members":{
            "Items":{"target":"n#Items"},"NextCursor":{"target":"smithy.api#String"}}},
        "n#Items":{"type":"list","member":{"target":"n#Thing"}},
        "n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
    }})
}

fn bound_paginator_endpoint()
-> Result<crate::endpoints::VerifiedEndpoint, Box<dyn std::error::Error>> {
    let properties = serde_json::json!({"authSchemes":[{"name":"sigv4","signingName":"example","signingRegion":"us-east-1"}]});
    let document = serde_json::json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://example.test","properties":properties}}]});
    let rules = crate::endpoints::RuleSet::parse(
        std::path::Path::new("paginator-binding-endpoint.json"),
        &document,
    )?;
    let cases: Vec<crate::endpoints::Case> = serde_json::from_value(
        serde_json::json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://example.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("example"))
        .map_err(|_| "fixture endpoint evidence failed".into())
}

fn compile_bound_paginator(
    protocol: &str,
    binding: serde_json::Value,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let model_dir = source.join("models/example/service/2020-01-01");
    std::fs::create_dir_all(&model_dir)?;
    std::fs::write(
        model_dir.join("model.json"),
        serde_json::to_vec(&bound_paginator_model(protocol, binding))?,
    )?;
    let plan = OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Items")?),
        identity: Some(Identity::Key(vec![MemberPath::new("Id")?])),
        paginator: Some(Paginator::Token(TokenPair::new(
            MemberPath::new("Cursor")?,
            MemberPath::new("NextCursor")?,
        ))),
        page_size: Some(10),
        ..OperationOverlay::default()
    };
    let policies = [ServiceOverlay::new("example", vec![("ListThings", plan)])?];
    let checked =
        crate::overlay::check_policies(&source, &policies, &crate::classes::DenyList::builtin())?;
    let fields = crate::smithy::Registry::parse(BOUND_REGISTRY)?;
    let base = crate::smithy::build_catalog(&source, &fields, "paginator-binding-fixture")?;
    let kinds = BOUND_KINDS.compile(&crate::names::Exceptions::none())?;
    let endpoints =
        std::collections::BTreeMap::from([("example".to_owned(), bound_paginator_endpoint()?)]);
    Ok(crate::runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

#[test]
fn given_document_protocol_http_bindings_when_catalog_is_loaded_then_pagination_uses_body()
-> TestResult {
    use inventory_core::catalog::runtime::{RuntimeCatalog, TokenPlacement};
    let fields = crate::smithy::Registry::parse(BOUND_REGISTRY)?;
    let cursor = fields.id("Cursor").ok_or("cursor field")?;
    let limit = fields.id("Limit").ok_or("limit field")?;
    let next = fields.id("NextCursor").ok_or("response field")?;
    [
        "aws.protocols#awsJson1_0",
        "aws.protocols#awsJson1_1",
        "smithy.protocols#rpcv2Cbor",
    ]
    .into_iter()
    .try_for_each(|protocol| {
        [
            serde_json::json!({"smithy.api#httpQuery":"cursor"}),
            serde_json::json!({"smithy.api#httpHeader":"X-Cursor"}),
            serde_json::json!({"smithy.api#httpLabel":{}}),
            serde_json::json!({"smithy.api#httpPayload":{}}),
        ]
        .into_iter()
        .try_for_each(|binding| -> TestResult {
            let bytes = compile_bound_paginator(protocol, binding)?;
            let catalog = RuntimeCatalog::from_bytes(&bytes)?;
            let plan = catalog
                .lookup("example", "ListThings")
                .ok_or("callable plan")?;
            assert_eq!(plan.pagination().tokens.len(), 1);
            let token = plan.pagination().tokens.first().ok_or("token plan")?;
            assert_eq!(token.request_path.as_ref(), &[cursor]);
            assert_eq!(
                token.request_placement,
                TokenPlacement::Body(vec![cursor].into_boxed_slice())
            );
            assert_eq!(token.response_path.as_ref(), &[next]);
            assert_eq!(
                token.response_placement,
                TokenPlacement::Body(vec![next].into_boxed_slice())
            );
            let size = plan
                .pagination()
                .page_size
                .as_ref()
                .ok_or("page size plan")?;
            assert_eq!(size.path.as_ref(), &[limit]);
            assert_eq!(
                size.placement,
                TokenPlacement::Body(vec![limit].into_boxed_slice())
            );
            assert_eq!(size.max, 10);
            Ok(())
        })
    })
}

#[test]
fn given_rest_json_query_bindings_when_catalog_is_loaded_then_pagination_keeps_query_names()
-> TestResult {
    use inventory_core::catalog::runtime::{RuntimeCatalog, TokenPlacement};
    let bytes = compile_bound_paginator(
        "aws.protocols#restJson1",
        serde_json::json!({"smithy.api#httpQuery":"cursor"}),
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("example", "ListThings")
        .ok_or("callable plan")?;
    let fields = crate::smithy::Registry::parse(BOUND_REGISTRY)?;
    let cursor = fields.id("Cursor").ok_or("cursor field")?;
    let limit = fields.id("Limit").ok_or("limit field")?;
    assert_eq!(plan.pagination().tokens.len(), 1);
    let token = plan.pagination().tokens.first().ok_or("token plan")?;
    assert_eq!(token.request_path.as_ref(), &[cursor]);
    assert_eq!(
        token.request_placement,
        TokenPlacement::Query("cursor".to_owned())
    );
    let size = plan
        .pagination()
        .page_size
        .as_ref()
        .ok_or("page size plan")?;
    assert_eq!(size.path.as_ref(), &[limit]);
    assert_eq!(size.placement, TokenPlacement::Query("limit".to_owned()));
    assert_eq!(size.max, 10);
    Ok(())
}

#[test]
fn given_endpoint_exception_when_encoded_then_fields_are_present() {
    use crate::runtime_catalog::encode::map;
    use inventory_core::record::{FieldId, Key, Value};

    let exception = crate::endpoints::EndpointException {
        region: "eu-west-1".to_owned(),
        url: "https://example.eu-west-1.amazonaws.com".to_owned(),
        signing_name: "example".to_owned(),
        signing_region: "eu-west-1".to_owned(),
    };
    let value = crate::runtime_catalog::plans::encode_endpoint_exception(&exception)
        .expect("encode_endpoint_exception");
    if let Value::Map(fields) = value {
        let region = fields
            .get(&Key::Field(FieldId::new(1).expect("field id")))
            .expect("region field");
        if let Value::Text(text) = region {
            assert_eq!(text, "eu-west-1");
        }
    }
}
