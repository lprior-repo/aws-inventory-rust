use super::{TestResult, call, endpoint, rules};
use serde_json::{Value, json};
use std::path::Path;

pub const REPORT: &str = concat!(
    "cases 11\nin_scope 4\npassed 3\nfailed 1\nskipped 7\n",
    "skipped other_partition 1\nskipped fips 1\nskipped dual_stack 1\n",
    "skipped custom_endpoint 1\nskipped other_parameters 1\nskipped operation_inputs 1\n",
    "skipped request_time_function 1\nservices 2\nglobal 1\nregional 1\n",
    "needs_overlay 0\nunsupported 0\n",
    "services_with_exceptions 0\nexceptions 0\n",
    "service global global https://global.example global-sign us-east-1\n",
    "service regional regional https://regional.{region}.example regional-sign {region}\n",
    "unsupported_case regional 8 parseURL\n"
);

pub fn fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    write_service(dir.path(), "regional", regional_rules(), cases())?;
    write_service(
        dir.path(),
        "global",
        global_rules(),
        json!([{
        "params":{"Region":"us-west-2"},"expect":{"endpoint":{
            "url":"https://global.example","properties":{"authSchemes":[{
                "name":"sigv4","signingRegion":"us-east-1"}]}}}}]),
    )?;
    Ok(dir)
}

pub fn write_service(root: &Path, name: &str, rules: Value, cases: Value) -> TestResult {
    let path = root.join(format!("models/{name}/service/2026-01-01/model.json"));
    let parent = path.parent().ok_or("no fixture parent")?;
    std::fs::create_dir_all(parent)?;
    let model = json!({"smithy":"2.0","shapes":{"example#Service":{"type":"service","traits":{
        "smithy.rules#endpointRuleSet":rules,"smithy.rules#endpointTests":{"testCases":cases},
        "aws.auth#sigv4":{"name":format!("{name}-sign")}}}}});
    std::fs::write(path, serde_json::to_vec(&model)?)?;
    Ok(())
}

fn parameters() -> Value {
    json!({"Region":{"type":"string"},"UseFIPS":{"type":"boolean","required":true,"default":false},
        "UseDualStack":{"type":"boolean","required":true,"default":false},
        "Endpoint":{"type":"string"},"Extra":{"type":"string"}})
}

pub fn global_rules() -> Value {
    rules(
        parameters(),
        json!([{"type":"endpoint","conditions":[],"endpoint":{
        "url":"https://global.example","properties":{"authSchemes":[{
            "name":"sigv4","signingRegion":"us-east-1"}]}}}]),
    )
}

pub fn regional_rules() -> Value {
    rules(
        parameters(),
        json!([
            endpoint(json!("unused"),json!([call("isSet",json!([{"ref":"Region"}])),
                call("stringEquals",json!([{"ref":"Region"},"us-overlay-1"])),
                call("parseURL",json!(["https://request-time"]))])),
            {"type":"endpoint","conditions":[call("isSet",json!([{"ref":"Region"}]))],
                "endpoint":{"url":"https://regional.{Region}.example","properties":{"authSchemes":[{
                    "name":"sigv4","signingRegion":"{Region}"}]}}},
            {"type":"error","conditions":[],"error":"Missing Region"}
        ]),
    )
}

fn cases() -> Value {
    let expected = json!({"endpoint":{"url":"https://regional.us-west-2.example",
        "properties":{"authSchemes":[{"name":"sigv4","signingRegion":"us-west-2"}]}}});
    json!([
        {"params":{"Region":"us-west-2"},"expect":expected},
        {"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"wrong"}}},
        {"params":{"Region":"cn-north-1"},"expect":{"endpoint":{"url":"unused"}}},
        {"params":{"Region":"us-east-1","UseFIPS":true},"expect":{"endpoint":{"url":"unused"}}},
        {"params":{"Region":"us-east-1","UseDualStack":true},"expect":{"endpoint":{"url":"unused"}}},
        {"params":{"Endpoint":"https://custom"},"expect":{"endpoint":{"url":"unused"}}},
        {"params":{"Extra":"value"},"expect":{"endpoint":{"url":"unused"}}},
        {"params":{"Region":"us-east-1"},"operationInputs":[{"operationName":"List"}],
            "expect":{"endpoint":{"url":"unused"}}},
        {"params":{"Region":"us-overlay-1"},"expect":{"endpoint":{"url":"unused"}}},
        {"expect":{"error":"Missing Region"}}
    ])
}
