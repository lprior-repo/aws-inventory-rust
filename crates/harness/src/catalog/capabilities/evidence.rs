use crate::catalog::json::Json;

use super::archive_name;

pub(super) fn document(rows: Vec<Json>, resources_sha256: &str, summary: Json) -> Json {
    Json::object(vec![
        ("schema_version", Json::count(1)),
        ("evidence", evidence(resources_sha256)),
        ("summary", summary),
        ("resource_types", Json::array(rows)),
    ])
}

fn evidence(resources_sha256: &str) -> Json {
    let mut fields = archive_fields(resources_sha256);
    fields.extend(policy_fields());
    Json::Object(fields)
}

fn archive_fields(resources_sha256: &str) -> Vec<(String, Json)> {
    vec![
        (
            "archive_file".to_owned(),
            Json::string(format!("upstream/{}", archive_name())),
        ),
        (
            "archive_sha256".to_owned(),
            Json::string(crate::catalog::ARCHIVE_SHA256),
        ),
        (
            "source_url".to_owned(),
            Json::string(
                "https://schema.cloudformation.us-east-1.amazonaws.com/CloudformationSchema.zip",
            ),
        ),
        ("catalog_sha256".to_owned(), Json::string(resources_sha256)),
    ]
}

fn policy_fields() -> Vec<(String, Json)> {
    vec![
        (
            "selection_policy".to_owned(),
            Json::string(
                "397 reviewed regional types across architectural service families; read and list handlers; empty list model accepted; documented parent-model exceptions excluded",
            ),
        ),
        (
            "warning".to_owned(),
            Json::string(
                "Schema eligibility, reviewed mappings and executed provider evidence are separate claims.",
            ),
        ),
    ]
}

pub fn not_run() -> Json {
    Json::object(vec![
        ("status", Json::string("not_run")),
        ("summary_file", Json::Null),
        ("list_attempted", Json::Bool(false)),
        ("get_attempted", Json::Bool(false)),
        ("read_observed", Json::Bool(false)),
        ("list_http_statuses", Json::object(Vec::new())),
        ("get_http_statuses", Json::object(Vec::new())),
    ])
}

pub fn without_executed_evidence(matrix: &mut Json) {
    if let Some(evidence) = matrix.get_mut("evidence") {
        evidence.remove("emulator");
    }
    if let Some(summary) = matrix.get_mut("summary") {
        summary.set("emulator_list_attempted", Json::count(0));
        summary.set("emulator_read_observed", Json::count(0));
        summary.remove("emulator_outcomes");
    }
    if let Some(rows) = matrix
        .get_mut("resource_types")
        .and_then(Json::as_array_mut)
    {
        for row in rows {
            row.set("emulator_verification", not_run());
        }
    }
}
