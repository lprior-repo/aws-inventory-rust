use std::collections::BTreeMap;

use aws_smithy_types::Document;
use inventory_core::{MAX_PROPERTY_BYTES, Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, now_millis, paginate, require_budget, required, sdk_error},
};

const CONFIG_QUERY: &str = "SELECT accountId, awsRegion, arn, resourceId, resourceName, resourceType, configuration, tags, configurationItemCaptureTime";

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests;

mod config;
pub(crate) use config::config;
mod explorer;
pub(crate) use explorer::explorer;
mod resource_types;
use resource_types::{explorer_resource_kind, json_tags, resource_kind, string};

#[cfg(test)]
use resource_types::{explorer_elbv2_kind, fallback_kind, normalize_resource_type};

#[cfg(test)]
use explorer::map_explorer;

#[cfg(test)]
use config::parse_config_row;
