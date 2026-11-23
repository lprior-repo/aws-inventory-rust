use super::*;
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
