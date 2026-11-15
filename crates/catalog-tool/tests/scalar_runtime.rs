use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::{catalog::runtime::{BindingSource, IdentityPlan, RuntimeCatalog}, inventory::ScalarType};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"},{"target":"n#GetThing"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"smithy.api#SCALAR"}},
"n#GetThing":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Thing"},"traits":{"smithy.api#http":{"method":"GET","uri":"/thing"}}},
"n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#SCALAR","traits":{"smithy.api#required":{}}}}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#SCALAR"}}}
}}"#;
const OVERLAY: &str = "format=1\nreviewed='2026-11-15'\nreviewer='fixture'\n[operation.ListThings]\nclass='enumerate'\nscope='region'\nitems='Items'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['$']\n[operation.GetThing]\nclass='expand'\nscope='region'\nitems='$'\nbind={Id={from='ListThings',path='$'}}\n[operation.GetThing.paginator]\nkind='none'\n[operation.GetThing.identity]\nkey=['Id']\n";
const KINDS: &str = "meta\tversion\t1\nmeta\treviewed\t2026-11-15\nmeta\treviewer\tfixture\napi\tAWS::ApiOnly::Scalar\tkind\tScalar fixture\napi\tAWS::ApiOnly::Detail\tkind\tDetail fixture\noperation\tthings\tListThings\tAWS::ApiOnly::Scalar\noperation\tthings\tGetThing\tAWS::ApiOnly::Detail\ndefault-unmapped\tFixture only\n";

fn endpoint() -> Result<endpoints::VerifiedEndpoint, Box<dyn std::error::Error>> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("scalar-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]))?;
    rules.run_cases(&cases)?.verified_endpoint(Some("things"))
        .map_err(|_| "fixture endpoint evidence failed".into())
}

fn compiled(shape: &str, identity: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let model_dir = source.join("models/things/service/2020-01-01");
    fs::create_dir_all(&model_dir)?;
    fs::create_dir_all(root.path().join("catalog/overlay"))?;
    fs::write(model_dir.join("model.json"), MODEL.replace("SCALAR", shape))?;
    fs::write(root.path().join("catalog/overlay/must-deny.txt"), "")?;
    fs::write(root.path().join("catalog/overlay/things.toml"), OVERLAY.replace("key=['$']", identity))?;
    let checked = overlay::check(root.path(), &source)?;
    let fields = smithy::Registry::parse("Id\nItems\n")?;
    let base = smithy::build_catalog(&source, &fields, "scalar-fixture")?;
    let kinds = kind_map::load(KINDS.as_bytes(), &names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("things".to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?)
}

#[test]
fn given_scalar_review_when_compiled_and_loaded_then_self_paths_and_typed_columns_roundtrip() -> TestResult {
    [("String", ScalarType::Text), ("Boolean", ScalarType::Bool), ("Byte", ScalarType::Integer),
        ("Short", ScalarType::Integer), ("Integer", ScalarType::Integer), ("Long", ScalarType::Integer)]
        .into_iter().try_for_each(|(shape, scalar)| {
            let given = compiled(shape, "key=['$']")?;
            let when = RuntimeCatalog::from_bytes(&given)?;
            let parent = when.lookup("things", "ListThings").ok_or("parent missing")?;
            assert!(matches!(parent.identity(), IdentityPlan::Key(paths) if paths.len() == 1 && paths.first().is_some_and(|path| path.is_empty())));
            let child = when.lookup("things", "GetThing").ok_or("child missing")?;
            assert!(matches!(&child.bindings().first().ok_or("binding missing")?.source, BindingSource::Operation { path, .. } if path.is_empty()));
            let schema = when.schemas().iter().find(|schema| schema.kind().as_str() == "AWS::ApiOnly::Scalar").ok_or("scalar schema missing")?;
            let column = schema.columns().first().ok_or("scalar column missing")?;
            assert_eq!((column.name(), column.scalar()), ("value", scalar));
            assert!(column.path().is_empty());
            Ok(())
        })
}

#[test]
fn given_scalar_arn_review_when_compiled_then_arn_and_root_structure_paths_stay_distinct() -> TestResult {
    let given = compiled("String", "arn='$'")?;
    let when = RuntimeCatalog::from_bytes(&given)?;
    assert!(matches!(when.lookup("things", "ListThings").ok_or("parent missing")?.identity(), IdentityPlan::Arn(path) if path.is_empty()));
    assert!(matches!(when.lookup("things", "GetThing").ok_or("child missing")?.identity(), IdentityPlan::Key(paths) if paths.first().is_some_and(|path| !path.is_empty())));
    Ok(())
}
