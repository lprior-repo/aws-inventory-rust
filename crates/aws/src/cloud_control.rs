//! One bounded, read-only Cloud Control adapter for the pinned type catalog.
//! Schema capabilities are evidence for a plan; live provider failures remain
//! explicit coverage failures. Raw provider properties are sanitized before
//! they become observations, SQL columns, tags, names, or relationship evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::{Arc, Mutex};

use aws_sdk_cloudcontrol::{
    Client,
    operation::{get_resource::GetResourceOutput, list_resources::ListResourcesOutput},
    types::ResourceDescription,
};
use inventory_core::{
    CollectionEvent, CollectionPage, CollectionScope, Edge, Resource, Selection, Source,
    validate_identifier,
};
use inventory_engine::limiter::ApiError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    AwsScanError,
    common::{
        Context, MAX_CHILDREN, PageBody, collect_bounded, error_category, now_millis, paginate,
        require_budget, sdk_error,
    },
};

mod parents;

const CATALOG_JSON: &str = include_str!("../../../catalog/cloudcontrol-resources.json");
const ROOT_TYPE_COUNT: usize = 397;
const TYPE_COUNT: usize = ROOT_TYPE_COUNT + parents::PARENT_TYPE_COUNT;
const TYPE_CONCURRENCY: usize = 4;
const READ_CONCURRENCY: usize = 4;
const MAX_LIST_ITEMS: usize = 1_000;
const MAX_IDENTIFIER_BYTES: usize = 1_024;
const MAX_PROVIDER_BYTES: usize = 262_144;
const MAX_LIST_BYTES: usize = 8_388_608;
const MAX_JSON_NODES: usize = 16_384;
const MAX_JSON_DEPTH: usize = 32;
const MAX_PATH_MATCHES: usize = 4_096;
const MAX_PATH_BYTES: usize = 1_048_576;
const MAX_RESOURCE_EDGES: usize = 4_096;
const MAX_RELATIONSHIP_RULES: usize = 256;
/// Cloud Control `ListResources` accepts a bounded page-size hint. The
/// collector's validated `--page-size` is clamped into the provider range so a
/// continuation token is requested instead of one unbounded list response.
const MAX_LIST_PAGE_SIZE: i32 = 100;

pub(super) fn list_page_size(ctx: &Context) -> i32 {
    ctx.page_size.clamp(1, MAX_LIST_PAGE_SIZE)
}

/// A reviewed provider-property reference; short EC2 IDs explicitly retain the
/// assumption that the owner is the scanning account, rather than claiming an
/// authoritative cross-account identity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub pointer: String,
    pub relation: String,
    pub target_kind: String,
    pub target_format: String,
    pub arn_template: Option<String>,
    pub evidence: String,
    /// Only exact returned ARNs of this service may satisfy a typed rule.
    #[serde(default)]
    pub arn_service: Option<String>,
    #[serde(default)]
    pub arn_resource_prefix: Option<String>,
    /// Reviewed single-ID grammars prevent names/compound identifiers from
    /// being inserted into an unrelated EC2 ARN template.
    #[serde(default)]
    pub identifier_prefix: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderColumn {
    pub name: String,
    pub pointer: String,
    pub json_type: String,
}

/// The complete catalog is available without making an AWS request. Optional
/// identity templates apply only to reviewed types and provider properties.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceType {
    pub type_name: String,
    pub kind: String,
    pub view_name: String,
    pub service: String,
    pub scope: String,
    pub home_region: Option<String>,
    pub read_supported: bool,
    pub list_supported: bool,
    pub list_requires_model: bool,
    pub list_model_validation: String,
    pub primary_identifier: Vec<String>,
    pub identifier_is_arn: bool,
    pub arn_pointers: Vec<String>,
    pub arn_template: Option<String>,
    pub arn_identifier_pointer: Option<String>,
    pub name_pointers: Vec<String>,
    pub write_only_properties: Vec<String>,
    pub sensitive_properties: Vec<String>,
    pub relationships: Vec<Relationship>,
    pub schema_file: String,
    pub schema_sha256: String,
    pub read_permissions: Vec<String>,
    pub list_permissions: Vec<String>,
    pub columns: Vec<ProviderColumn>,
    #[serde(skip)]
    sensitive_names: Arc<BTreeSet<String>>,
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    evidence: CatalogEvidence,
    resource_types: Vec<ResourceType>,
}

#[derive(Deserialize)]
struct CatalogEvidence {
    global_sensitive_property_names: BTreeSet<String>,
}

#[derive(Default)]
struct IdentityLedger {
    hashes: BTreeSet<[u8; 32]>,
}

#[cfg(test)]
mod architecture_tests;

#[cfg(test)]
mod tests;

mod catalog_validation;
pub use catalog_validation::{
    catalog, global_sensitive_names, parent_catalog, root_catalog, validate_selection,
};
use catalog_validation::{invalid, validate_descriptor, validate_pointer};
mod collection;
pub(crate) use collection::collect;
mod universal;
use collection::{
    collect_type_with, emit_child, identifier, record_unsupported, remember_list_page,
    validate_response_type,
};
pub use universal::{UniversalReport, digest as universal_digest, report as universal_report};
mod property_paths;
use property_paths::{expand_pointer, map_get, sanitized_properties};
mod identity;
use identity::{arn_parts, expand_arn, map_description};
mod relationships;
use relationships::map_relationships;

#[cfg(test)]
use relationships::{reference_arn_matches, reference_identifier_matches};
