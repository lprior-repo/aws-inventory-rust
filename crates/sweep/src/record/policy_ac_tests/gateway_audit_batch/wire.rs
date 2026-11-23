use super::*;

pub(super) fn assert_wire(
    family: &Family,
    case: &ChildCase,
    wire: &http::Request<bytes::Bytes>,
    parent_value: &str,
    resumed: bool,
) -> TestResult {
    if family.service == "batch" {
        let body: Value = serde_json::from_slice(wire.body())?;
        assert_eq!(
            body.get("jobQueue").and_then(Value::as_str),
            Some(parent_value)
        );
        if case.operation == "ListJobs" {
            assert_eq!(
                body.get("filters"),
                Some(&json!([{"name":"JOB_NAME","values":["*"]}]))
            );
            assert_eq!(body.get("jobStatus"), None);
            assert_eq!(body.get("arrayJobId"), None);
            assert_eq!(body.get("multiNodeJobId"), None);
        }
        assert_eq!(
            body.get("nextToken").and_then(Value::as_str),
            resumed.then_some("later")
        );
    } else {
        assert!(
            wire.uri()
                .path()
                .contains(&format!("/apis/{parent_value}/"))
                || wire
                    .uri()
                    .path()
                    .contains(&format!("/restapis/{parent_value}/"))
        );
        let query = wire.uri().query().unwrap_or("");
        assert!(!query.contains("embed"));
        assert_eq!(query.contains("later"), resumed);
    }
    Ok(())
}
