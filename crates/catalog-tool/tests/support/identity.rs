use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Input":{"type":"structure","members":{"Mode":{"target":"n#Choice","traits":{"smithy.api#required":{},"smithy.api#httpQuery":"mode"}},"Modes":{"target":"n#Choices"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"ITEM_TARGET"}},
"n#Thing":{"type":"structure","members":{"Name":{"target":"n#Choice"},"Count":{"target":"smithy.api#Long"},"Flag":{"target":"smithy.api#Boolean"}}},
"n#Choice":{"type":"enum","members":{"ACTIVE":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"active-wire"}},"SECOND":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"second-wire"}}}},
"n#Choices":{"type":"list","member":{"target":"n#Choice"}}
}}"#;

pub fn compiled(identity: &str, fixed: &str, scalar: bool) -> TestResult<Vec<u8>> {
    compiled_fixed(identity, &format!("Mode='{fixed}'"), scalar)
}

pub fn compiled_singleton(identity: &str) -> TestResult<Vec<u8>> {
    compile_fixture(identity, "Mode='active-wire'", false, true)
}

pub fn compiled_fixed(identity: &str, fixed: &str, scalar: bool) -> TestResult<Vec<u8>> {
    compile_fixture(identity, fixed, scalar, false)
}

fn compile_fixture(identity: &str, fixed: &str, scalar: bool, singleton: bool) -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let model_dir = source.join("models/things/service/2020-01-01");
    fs::create_dir_all(&model_dir)?;
    fs::create_dir_all(root.path().join("catalog/overlay"))?;
    let target = if scalar { "n#Choice" } else { "n#Thing" };
    let model = MODEL.replace("ITEM_TARGET", target);
    let model = if singleton {
        model.replace("\"Items\":{\"target\":\"n#Items\"}", "\"Items\":{\"target\":\"n#Thing\"}")
    } else {
        model
    };
    fs::write(model_dir.join("model.json"), model)?;
    fs::write(root.path().join("catalog/overlay/must-deny.txt"), "")?;
    fs::write(root.path().join("catalog/overlay/things.toml"), format!(
        "format=1\nreviewed='2026-11-15'\nreviewer='identity-fixture'\n[operation.ListThings]\nclass='enumerate'\nscope='region'\nitems='Items'\nfixed={{{fixed}}}\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\n{identity}\n"
    ))?;
    let checked = overlay::check(root.path(), &source)?;
    let fields = smithy::Registry::parse("Count\nFlag\nItems\nMode\nModes\nName\n")?;
    let base = smithy::build_catalog(&source, &fields, "identity-fixture")?;
    let kinds = kind_map::load(
        b"meta\tversion\t1\nmeta\treviewed\t2026-11-15\nmeta\treviewer\tfixture\napi\tAWS::ApiOnly::Identity\tkind\tIdentity fixture\noperation\tthings\tListThings\tAWS::ApiOnly::Identity\ndefault-unmap\tFixture only\n",
        &names::Exceptions::none(),
    )?;
    let endpoints = BTreeMap::from([("things".to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?)
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("identity-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]))?;
    rules.run_cases(&cases)?.verified_endpoint(Some("things"))
        .map_err(|failure| format!("fixture endpoint evidence failed: {failure:?}").into())
}
