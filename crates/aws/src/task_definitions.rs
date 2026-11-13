//! ACTIVE ECS task-definition revisions, with separate list/description coverage.
//! SDK responses may contain environment values; only explicit reference and
//! architecture fields below are retained in inventory events.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::sync::Mutex;

use aws_sdk_ecs::{
    Client,
    operation::{
        describe_task_definition::DescribeTaskDefinitionOutput,
        list_task_definitions::ListTaskDefinitionsOutput,
    },
    types::{
        ContainerDefinition, LogConfiguration, Secret, SortOrder, TaskDefinitionField,
        TaskDefinitionStatus, Volume,
    },
};
use aws_smithy_types::DateTime;
use inventory_core::{Edge, Resource, Source, validate_identifier};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{
        Context, MAX_CHILDREN, PageBody, collect_bounded, paginate, require_budget, required,
        sdk_error,
    },
    ecs::{merge_reference_edge, tags},
};

const DESCRIBE_CONCURRENCY: usize = 4;
const MAX_IDENTITY_BYTES: usize = 8_388_608;
const MAX_COMPONENTS: usize = 4_096;
const LIST_SCOPE: &str = "aws_ecs_task_definition_list";
const DEFINITION_KIND: &str = "aws_ecs_task_definition";

/// The only scan-wide state is a bounded identity ledger. No SDK definitions or
/// mapped resources accumulate across pages; descriptions stream independently.
#[derive(Default)]
struct ListedIdentities {
    arns: BTreeSet<String>,
    bytes: usize,
}

struct ArnParts<'a> {
    service: &'a str,
    region: &'a str,
    account: &'a str,
    resource: &'a str,
}

struct DefinitionIdentity<'a> {
    account: &'a str,
    region: &'a str,
    family: &'a str,
    revision: i32,
}

#[cfg(test)]
mod tests;

mod collection;
pub(crate) use collection::collect;
mod identity;
use identity::{arn_parts, consume, definition_identity, millis, option_keys, secret_references};
mod components;
use components::{map_container, map_volume};
mod relationships;
use relationships::{file_system_edge, returned_arn_edge};
mod definition;
use definition::map_definition;

#[cfg(test)]
use collection::{collect_with, remember_list_page};
