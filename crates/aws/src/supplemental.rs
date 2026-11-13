//! Optional, existing indexes are supplementary evidence, not completeness proofs.

use std::collections::BTreeMap;

use aws_smithy_types::Document;
use inventory_core::{MAX_PROPERTY_BYTES, Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, now_millis, paginate, require_budget, required, sdk_error},
};

const CONFIG_QUERY: &str = "SELECT accountId, awsRegion, arn, resourceId, resourceName, resourceType, configuration, tags, configurationItemCaptureTime";

// Config keeps its original nested representation under `configuration`. Only
// scalar fields and arrays of strings have the same shape as these native
// aliases. In particular, camelCase routes/actions must not become partial
// snake_case native evidence, and missing fields must not become empty lists.

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
