pub mod endpoints_support;

use catalog_tool::endpoints::{Case, Error, Fault, RuleSet, build_report};
use endpoints_support::report::{global_rules, write_service};
use endpoints_support::{TestResult, endpoint, result, rules};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

#[test]
fn case_count_exactly_20000_succeeds_and_20001_fails() -> TestResult {
    let document = rules(
        json!({}),
        json!([endpoint(json!("https://bounds.test"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("cases.json"), &document)?;
    let mut cases: Vec<Case> = (0..20_000)
        .map(|_| Case {
            params: BTreeMap::new(),
            expect: result("https://bounds.test"),
            operation_inputs: Vec::new(),
        })
        .collect();
    let counts = parsed.run_cases(&cases)?;
    assert_eq!(
        (counts.in_scope(), counts.passed(), counts.failed()),
        (20_000, 20_000, 0)
    );
    assert!(counts.skipped().is_empty());
    cases.push(Case {
        params: BTreeMap::new(),
        expect: result("https://bounds.test"),
        operation_inputs: Vec::new(),
    });
    assert!(matches!(parsed.run_cases(&cases),
        Err(Error::Model { file, kind: Fault::Bound("test cases") })
            if file == Path::new("cases.json")));
    Ok(())
}

#[test]
fn service_count_exactly_1024_succeeds_and_1025_fails() -> TestResult {
    let dir = tempfile::tempdir()?;
    for index in 0..1024 {
        write_service(
            dir.path(),
            &format!("service-{index:04}"),
            global_rules(),
            json!([
                {"params":{"Region":"us-west-2"},"expect":{"endpoint":{
                    "url":"https://global.example","properties":{"authSchemes":[{
                        "name":"sigv4","signingRegion":"us-east-1"}]}}}}
            ]),
        )?;
    }
    let report = build_report(dir.path())?;
    assert!(report.starts_with("cases 1024\nin_scope 1024\npassed 1024\nfailed 0\nskipped 0\n"));
    assert!(
        report.contains("services 1024\nglobal 1024\nregional 0\nneeds_overlay 0\nunsupported 0\n")
    );
    assert_eq!(
        report
            .lines()
            .filter(|line| line.starts_with("service "))
            .count(),
        1024
    );
    write_service(dir.path(), "overflow", global_rules(), json!([]))?;
    assert!(matches!(build_report(dir.path()),
        Err(Error::Model { file, kind: Fault::Bound("services") })
            if file == dir.path().join("models")));
    Ok(())
}
