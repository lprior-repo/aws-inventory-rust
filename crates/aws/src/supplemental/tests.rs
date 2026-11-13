use super::{map_explorer, parse_config_row, resource_kind};
use aws_smithy_types::{DateTime, Document};
use inventory_core::Source;
use serde_json::{Value, json};

fn config_fixture(resource_type: &str, configuration: &Value) -> String {
    json!({
        "accountId": "222222222222", "awsRegion": "eu-west-1",
        "resourceType": resource_type, "resourceId": "recorded-id",
        "configuration": configuration, "configurationItemCaptureTime": "2026-11-12T00:00:00Z",
    })
    .to_string()
}

fn explorer_fixture(resource_type: &str, arn: &str) -> aws_sdk_resourceexplorer2::types::Resource {
    aws_sdk_resourceexplorer2::types::Resource::builder()
        .arn(arn)
        .owning_account_id("222222222222")
        .region("eu-west-1")
        .resource_type(resource_type)
        .build()
}

mod config_maps_all_six_network_kinds;
mod config_projects_recorded_elbv2_scalars_without_native_enrichment;
mod explorer_keeps_owner_region_and_source_freshness;
mod explorer_maps_verified_v2_arns_including_ambiguous_type;
