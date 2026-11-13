//! One reviewed parent binding plan row.

use crate::catalog::json::Json;

pub(super) fn plan_row(name: &str, parent_name: &str, source: &str, target: &str) -> Json {
    Json::object(vec![
        ("type_name", Json::string(name)),
        ("parent_type", Json::string(parent_name)),
        (
            "bindings",
            Json::array(vec![Json::object(vec![
                ("source_pointer", Json::string(format!("/{source}"))),
                ("model_property", Json::string(target)),
            ])]),
        ),
        ("relation", Json::string("CHILD_OF")),
        (
            "evidence",
            Json::string("reviewed_parent_resource_model_and_matching_read_properties"),
        ),
    ])
}
