use crate::catalog::json::Json;
use crate::catalog::rules::GLOBAL_SENSITIVE;
use crate::catalog::{ARCHIVE, ARCHIVE_SHA256};

use super::build::{Plans, Resources};

pub(super) fn document(root_sha: &str, resources: Resources, plans: Plans) -> Json {
    Json::object(vec![
        ("schema_version", Json::count(1)),
        ("evidence", evidence(root_sha)),
        ("resource_types", rows(resources)),
        ("parent_plans", rows(plans)),
    ])
}

fn rows(rows: Resources) -> Json {
    Json::array(rows.into_iter().map(|(_, row)| row).collect())
}

fn evidence(root_sha: &str) -> Json {
    let mut fields = source_fields(root_sha);
    fields.extend(url_fields());
    fields.extend(review_fields());
    Json::Object(fields)
}

fn source_fields(root_sha: &str) -> Vec<(String, Json)> {
    vec![
        (
            "source_kind".to_owned(),
            Json::string("official_aws_cloudformation_registry_schema_archive"),
        ),
        (
            "archive_file".to_owned(),
            Json::string(format!("upstream/{}", archive_name())),
        ),
        ("archive_sha256".to_owned(), Json::string(ARCHIVE_SHA256)),
        ("root_catalog_sha256".to_owned(), Json::string(root_sha)),
        ("global_sensitive_property_names".to_owned(), global_names()),
    ]
}

fn url_fields() -> Vec<(String, Json)> {
    vec![
        (
            "list_semantics_url".to_owned(),
            Json::string(
                "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html",
            ),
        ),
        (
            "list_api_url".to_owned(),
            Json::string(
                "https://docs.aws.amazon.com/cloudcontrolapi/latest/APIReference/API_ListResources.html",
            ),
        ),
    ]
}

fn review_fields() -> Vec<(String, Json)> {
    vec![
        (
            "scope_review".to_owned(),
            Json::string(
                "Six reviewed regional root types; one dependency level; exact string property bindings; reviewed high-confidence candidate additions",
            ),
        ),
        ("live_validated".to_owned(), Json::Bool(false)),
        ("limitations".to_owned(), limitations()),
    ]
}

fn global_names() -> Json {
    Json::array(
        GLOBAL_SENSITIVE
            .iter()
            .map(|name| Json::string(*name))
            .collect(),
    )
}

fn limitations() -> Json {
    Json::array(vec![
        Json::string("Schema and request-model audit is not live AWS provider validation."),
        Json::string(
            "Two-level parents, account/global contexts, and other unreviewed bindings remain excluded.",
        ),
        Json::string(
            "Parent discovery uses complete sanitized GetResource properties; missing context records a failure.",
        ),
    ])
}

fn archive_name() -> &'static str {
    ARCHIVE.rsplit('/').next().unwrap_or(ARCHIVE)
}
