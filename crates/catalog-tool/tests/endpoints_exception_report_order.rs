pub mod endpoints_support;

use catalog_tool::endpoints::build_report;
use endpoints_support::exceptions::{case, route};
use endpoints_support::report::write_service;
use endpoints_support::{TestResult, call, rules};
use serde_json::json;

#[test]
fn complete_report_bytes_order_two_exception_rows_by_region_not_input_order() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
            route(
                "https://global.test",
                json!([call("stringEquals", json!([{"ref":"Region"},"aws-global"]))]),
                "us-east-1"
            ),
            route(
                "https://africa.test",
                json!([call("stringEquals", json!([{"ref":"Region"},"af-south-1"]))]),
                "us-east-1"
            ),
            route("https://svc.{Region}.test", json!([]), "{Region}")
        ]),
    );
    let cases = json!([
        case("aws-global", "https://global.test", "us-east-1"),
        case("af-south-1", "https://africa.test", "us-east-1"),
        case("us-west-2", "https://svc.us-west-2.test", "us-west-2")
    ]);
    let dir = tempfile::tempdir()?;
    write_service(dir.path(), "ordered", document, cases)?;

    assert_eq!(
        build_report(dir.path())?.as_bytes(),
        concat!(
            "cases 3\nin_scope 3\npassed 3\nfailed 0\nskipped 0\n",
            "skipped other_partition 0\nskipped fips 0\nskipped dual_stack 0\n",
            "skipped custom_endpoint 0\nskipped other_parameters 0\nskipped operation_inputs 0\n",
            "skipped request_time_function 0\nservices 1\nglobal 0\nregional 0\n",
            "needs_overlay 1\nunsupported 0\nservices_with_exceptions 1\nexceptions 2\n",
            "service ordered needs-overlay https://svc.{region}.test ordered-sign {region} ",
            "reason:\"regional template contradicted by more published cases than it matches\"\n",
            "exception ordered af-south-1 https://africa.test ordered-sign us-east-1\n",
            "exception ordered aws-global https://global.test ordered-sign us-east-1\n"
        )
        .as_bytes()
    );
    Ok(())
}
