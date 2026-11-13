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
    CollectionEvent, CollectionPage, CollectionScope, Edge, Resource, Source, validate_identifier,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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

fn invalid(message: impl Into<String>) -> AwsScanError {
    AwsScanError::InvalidData(message.into())
}

/// Parse and validate the embedded, pinned catalog, without credentials or I/O.
pub fn root_catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    let mut catalog: Catalog = serde_json::from_str(CATALOG_JSON)
        .map_err(|error| invalid(format!("embedded Cloud Control catalog: {error}")))?;
    if catalog.schema_version != 1 || catalog.resource_types.len() != ROOT_TYPE_COUNT {
        return Err(invalid(
            "unexpected Cloud Control catalog version or type count",
        ));
    }
    let mut types = BTreeSet::new();
    let mut views = BTreeSet::new();
    let sensitive = Arc::new(catalog.evidence.global_sensitive_property_names);
    if sensitive.is_empty() || sensitive.len() > 128 {
        return Err(invalid("invalid common sensitive-property policy"));
    }
    for descriptor in &mut catalog.resource_types {
        validate_descriptor(descriptor, false)?;
        if !types.insert(descriptor.type_name.clone())
            || !views.insert(descriptor.view_name.clone())
        {
            return Err(invalid("duplicate Cloud Control type or SQL view"));
        }
        descriptor.sensitive_names = Arc::clone(&sensitive);
    }
    Ok(catalog.resource_types)
}

/// Reviewed parent-dependent types, kept separate from the frozen root catalog.
pub fn parent_catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    parents::catalog()
}

/// Complete runnable catalog, including types that need a discovered parent.
pub fn catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    let mut descriptors = root_catalog()?;
    descriptors.extend(parent_catalog()?);
    parents::validate_catalog(&descriptors)?;
    Ok(descriptors)
}

/// An empty selection means the whole pinned catalog. Unknown selections fail
/// before credentials are loaded; duplicate selectors are harmlessly deduped.
pub fn validate_selection(selected: &[String]) -> Result<Vec<ResourceType>, AwsScanError> {
    if selected.len() > TYPE_COUNT {
        return Err(AwsScanError::InvalidOptions(
            "at most 418 Cloud Control resource types may be selected".into(),
        ));
    }
    let descriptors = catalog()?;
    let mut requested: BTreeSet<&str> = selected.iter().map(String::as_str).collect();
    for name in &requested {
        if !descriptors
            .iter()
            .any(|descriptor| descriptor.type_name == *name)
        {
            return Err(AwsScanError::InvalidOptions(format!(
                "resource type is not in the pinned Cloud Control catalog: {}",
                name.chars().take(196).collect::<String>()
            )));
        }
    }
    let plans = parents::plans()?;
    if !requested.is_empty() {
        for plan in &plans {
            if requested.contains(plan.type_name.as_str()) {
                requested.insert(plan.parent_type.as_str());
            }
        }
    }
    Ok(descriptors
        .into_iter()
        .filter(|descriptor| {
            requested.is_empty() || requested.contains(descriptor.type_name.as_str())
        })
        .collect())
}

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_pointer(pointer: &str) -> Result<(), AwsScanError> {
    if !pointer.starts_with('/')
        || pointer.len() > 1_024
        || pointer.split('/').count() > MAX_JSON_DEPTH
        || pointer.contains('\0')
    {
        return Err(invalid("invalid or excessive catalog property pointer"));
    }
    Ok(())
}

fn validate_descriptor(
    descriptor: &ResourceType,
    parent_required: bool,
) -> Result<(), AwsScanError> {
    validate_identifier(&descriptor.type_name, "Cloud Control type")?;
    if !descriptor.type_name.starts_with("AWS::")
        || descriptor.type_name.len() > 196
        || !safe_name(&descriptor.kind)
        || !safe_name(&descriptor.view_name)
        || !descriptor.view_name.starts_with("aws_cc_")
        || descriptor.scope != "regional"
        || descriptor.home_region.is_some()
        || !descriptor.read_supported
        || !descriptor.list_supported
        || descriptor.list_requires_model != parent_required
        || descriptor.primary_identifier.is_empty()
        || descriptor.primary_identifier.len() > 16
        || descriptor.columns.len() > 200
        || descriptor.relationships.len() > MAX_RELATIONSHIP_RULES
        || descriptor.write_only_properties.len() > 256
        || descriptor.sensitive_properties.len() > 256
        || descriptor.schema_sha256.len() != 64
        || !descriptor
            .schema_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid(format!(
            "invalid Cloud Control descriptor {}",
            descriptor.type_name
        )));
    }
    for pointer in descriptor
        .primary_identifier
        .iter()
        .chain(&descriptor.arn_pointers)
        .chain(&descriptor.name_pointers)
        .chain(&descriptor.write_only_properties)
        .chain(&descriptor.sensitive_properties)
        .chain(descriptor.arn_identifier_pointer.iter())
    {
        validate_pointer(pointer)?;
    }
    let mut columns = BTreeSet::new();
    for column in &descriptor.columns {
        validate_pointer(&column.pointer)?;
        if !safe_name(&column.name) || !columns.insert(&column.name) {
            return Err(invalid("invalid or duplicate provider column"));
        }
    }
    for relation in &descriptor.relationships {
        validate_pointer(&relation.pointer)?;
        validate_identifier(&relation.relation, "relation")?;
        if !safe_name(&relation.target_kind)
            || !matches!(relation.target_format.as_str(), "arn" | "template")
            || (relation.target_format == "template" && relation.arn_template.is_none())
        {
            return Err(invalid("invalid relationship descriptor"));
        }
        for constraint in [
            &relation.arn_service,
            &relation.arn_resource_prefix,
            &relation.identifier_prefix,
        ] {
            if constraint.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > 128 || value.chars().any(char::is_control)
            }) {
                return Err(invalid("invalid relationship identity constraint"));
            }
        }
        if (relation.target_format == "arn" && relation.identifier_prefix.is_some())
            || (relation.target_format == "template"
                && (relation.arn_service.is_some() || relation.arn_resource_prefix.is_some()))
        {
            return Err(invalid(
                "relationship constraint does not match its identity format",
            ));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all, fields(resource_types = types.len()))]
pub(crate) async fn collect(ctx: &Context, types: &[ResourceType]) -> Result<(), AwsScanError> {
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    if types.len() > TYPE_COUNT {
        return Err(invalid("Cloud Control type bound exceeded"));
    }
    let groups = parents::groups(types)?;
    let client = Client::new(&ctx.config);
    collect_bounded(&groups, TYPE_CONCURRENCY, |group| {
        let client = &client;
        async move {
            if ctx.executor.is_cancelled() {
                return Err(ApiError::Cancelled.into());
            }
            parents::collect_group_with(
                ctx,
                group,
                |request: parents::ListRequest| async move {
                    ctx.execute("cloudcontrol", "read", || async {
                        client
                            .list_resources()
                            .type_name(&request.type_name)
                            .set_resource_model(request.resource_model.clone())
                            .set_next_token(request.token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await
                },
                |request: parents::GetRequest| async move {
                    ctx.execute("cloudcontrol", "read", || async {
                        client
                            .get_resource()
                            .type_name(&request.type_name)
                            .identifier(&request.identifier)
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await
                },
            )
            .await
        }
    })
    .await?;
    Ok(())
}

#[derive(Default)]
struct IdentityLedger {
    hashes: BTreeSet<[u8; 32]>,
}

fn identifier(description: &ResourceDescription) -> Result<&str, AwsScanError> {
    let value = description
        .identifier()
        .ok_or_else(|| invalid("Cloud Control response lacks Identifier"))?;
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.contains('\0') {
        return Err(invalid(
            "Cloud Control Identifier is empty or exceeds its 1024-byte model bound",
        ));
    }
    Ok(value)
}

fn validate_response_type(expected: &str, actual: Option<&str>) -> Result<(), AwsScanError> {
    if actual.is_some_and(|actual| actual != expected) {
        return Err(invalid("Cloud Control returned a different resource type"));
    }
    Ok(())
}

fn remember_list_page(
    descriptor: &ResourceType,
    output: &ListResourcesOutput,
    ledger: &Mutex<IdentityLedger>,
    budget: usize,
) -> Result<(), AwsScanError> {
    validate_response_type(&descriptor.type_name, output.type_name())?;
    let descriptions = output.resource_descriptions();
    require_budget(descriptions.len(), budget)?;
    if descriptions.len() > MAX_LIST_ITEMS {
        return Err(invalid(
            "Cloud Control service page exceeds the local 1000-item bound",
        ));
    }
    if output
        .next_token()
        .is_some_and(|token| token.is_empty() || token.len() > 4_096)
    {
        return Err(invalid(
            "Cloud Control returned an invalid continuation token",
        ));
    }
    let mut page = BTreeSet::new();
    let mut bytes = 0_usize;
    for description in descriptions {
        let id = identifier(description)?;
        let digest: [u8; 32] = Sha256::digest(id.as_bytes()).into();
        if !page.insert(digest) {
            return Err(invalid("Cloud Control listed the same identifier twice"));
        }
        let properties_bytes = description.properties().map_or(0, str::len);
        bytes = bytes
            .checked_add(properties_bytes)
            .ok_or_else(|| invalid("list property byte overflow"))?;
        if properties_bytes > MAX_PROVIDER_BYTES || bytes > MAX_LIST_BYTES {
            return Err(invalid(
                "Cloud Control list properties exceed the local byte budget",
            ));
        }
    }
    let mut ledger = ledger
        .lock()
        .map_err(|_| invalid("Cloud Control identity ledger was poisoned"))?;
    if ledger.hashes.len().saturating_add(page.len()) > MAX_CHILDREN {
        return Err(AwsScanError::ChildLimit);
    }
    if page.iter().any(|id| ledger.hashes.contains(id)) {
        return Err(invalid("Cloud Control repeated an identifier across pages"));
    }
    ledger.hashes.extend(page);
    Ok(())
}

#[tracing::instrument(skip_all, fields(resource_type = descriptor.type_name))]
async fn collect_type_with<L, LFut, G, GFut>(
    ctx: &Context,
    descriptor: &ResourceType,
    list: L,
    get: G,
) -> Result<(), AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(String) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let ledger = Mutex::new(IdentityLedger::default());
    paginate(
        ctx,
        ctx.scope(
            "cloud_control_list",
            Some(&descriptor.type_name),
            Source::CloudControl,
        ),
        |token, budget| {
            let list = &list;
            let get = &get;
            let ledger = &ledger;
            async move {
                if ctx.executor.is_cancelled() {
                    return Err(ApiError::Cancelled.into());
                }
                let output = list(token).await?;
                remember_list_page(descriptor, &output, ledger, budget)?;
                collect_bounded(
                        output.resource_descriptions(),
                        READ_CONCURRENCY,
                        |description| async move {
                            enrich_with(ctx, descriptor, description, get).await
                        },
                    )
                    .await?;
                Ok(PageBody {
                    listed_count: output.resource_descriptions().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await?;
    if ctx.executor.is_cancelled() {
        Err(ApiError::Cancelled.into())
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all, fields(resource_type = descriptor.type_name))]
async fn enrich_with<G, GFut>(
    ctx: &Context,
    descriptor: &ResourceType,
    listed: &ResourceDescription,
    get: &G,
) -> Result<(), AwsScanError>
where
    G: Fn(String) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let listed_identifier = identifier(listed)?;
    let parent = format!(
        "{}:{:x}",
        descriptor.type_name,
        Sha256::digest(listed_identifier.as_bytes())
    );
    let scope = ctx.scope("cloud_control_get", Some(&parent), Source::CloudControl);
    let mapped = match get(listed_identifier.to_owned()).await {
        Ok(output) => map_get(ctx, descriptor, listed_identifier, &output),
        Err(error) => Err(error.into()),
    };
    let outcome = match mapped {
        Ok(body) => emit_child(ctx, &scope, body).await,
        Err(error) => Err(error),
    };
    match outcome {
        Ok(()) => Ok(()),
        Err(AwsScanError::ChannelClosed) => Err(AwsScanError::ChannelClosed),
        Err(AwsScanError::Api(ApiError::Cancelled)) => Err(ApiError::Cancelled.into()),
        Err(error) => {
            // Queue known failure first. Store retains its failed flag when a
            // later thin page is committed in this same child scope.
            ctx.failure(&scope, error_category(&error), error.to_string())
                .await?;
            let fallback = match map_description(ctx, descriptor, listed_identifier, listed, false)
            {
                Ok(body) => emit_child(ctx, &scope, body).await,
                Err(error) => Err(error),
            };
            match fallback {
                Ok(()) => {}
                Err(AwsScanError::ChannelClosed) => return Err(AwsScanError::ChannelClosed),
                Err(fallback_error) => {
                    ctx.failure(
                        &scope,
                        "invalid_response",
                        format!("list-only fallback could not be retained: {fallback_error}"),
                    )
                    .await?;
                }
            }
            Ok(())
        }
    }
}

#[tracing::instrument(skip_all)]
async fn emit_child(
    ctx: &Context,
    scope: &CollectionScope,
    body: PageBody,
) -> Result<(), AwsScanError> {
    let page = CollectionPage {
        scope: scope.clone(),
        page_number: 1,
        next_token: None,
        resources: body.resources,
        edges: body.edges,
    };
    page.validate()?;
    ctx.sender
        .send(CollectionEvent::Page(page))
        .await
        .map_err(|_| AwsScanError::ChannelClosed)?;
    tokio::task::consume_budget().await;
    Ok(())
}

fn map_get(
    ctx: &Context,
    descriptor: &ResourceType,
    listed_identifier: &str,
    output: &GetResourceOutput,
) -> Result<PageBody, AwsScanError> {
    validate_response_type(&descriptor.type_name, output.type_name())?;
    let description = output
        .resource_description()
        .ok_or_else(|| invalid("GetResource response lacks ResourceDescription"))?;
    if identifier(description)? != listed_identifier {
        return Err(invalid("GetResource returned a different identifier"));
    }
    if description.properties().is_none() {
        return Err(invalid("GetResource response lacks Properties"));
    }
    map_description(ctx, descriptor, listed_identifier, description, true)
}

fn escape_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn normalized_key(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Expand a reviewed wildcard pointer without recursion. Both arrays and
/// string-keyed property maps are supported; array positions never shift.
fn expand_pointer(root: &Value, pattern: &str) -> Result<Vec<String>, AwsScanError> {
    validate_pointer(pattern)?;
    let mut paths = vec![String::new()];
    for segment in pattern.split('/').skip(1) {
        let mut next = Vec::new();
        let mut bytes = 0_usize;
        for parent in paths {
            let Some(value) = root.pointer(&parent) else {
                continue;
            };
            let parts: Vec<String> = if segment == "*" {
                match value {
                    Value::Array(values) => values
                        .iter()
                        .enumerate()
                        .map(|(index, _)| index.to_string())
                        .collect(),
                    Value::Object(values) => values.keys().map(|key| escape_pointer(key)).collect(),
                    _ => Vec::new(),
                }
            } else {
                vec![segment.to_owned()]
            };
            for part in parts {
                let path = format!("{parent}/{part}");
                if root.pointer(&path).is_some() {
                    bytes = bytes
                        .checked_add(path.len())
                        .ok_or_else(|| invalid("property path byte overflow"))?;
                    if next.len() >= MAX_PATH_MATCHES || bytes > MAX_PATH_BYTES {
                        return Err(invalid("property pointer expansion exceeds its bound"));
                    }
                    next.push(path);
                }
            }
        }
        paths = next;
    }
    Ok(paths)
}

fn common_redactions(
    root: &Value,
    sensitive: &BTreeSet<String>,
) -> Result<BTreeSet<String>, AwsScanError> {
    let mut paths = BTreeSet::new();
    let mut stack = vec![(String::new(), root, 0_usize)];
    let mut nodes = 0_usize;
    let mut path_bytes = 0_usize;
    for _ in 0..MAX_JSON_NODES {
        let Some((path, value, depth)) = stack.pop() else {
            return Ok(paths);
        };
        nodes = nodes
            .checked_add(1)
            .ok_or_else(|| invalid("JSON node counter overflow"))?;
        if depth > MAX_JSON_DEPTH || path.len() > MAX_PATH_BYTES {
            return Err(invalid("provider JSON nesting exceeds its bound"));
        }
        let next_depth = depth
            .checked_add(1)
            .ok_or_else(|| invalid("JSON depth overflow"))?;
        match value {
            Value::Object(values) => {
                for (key, child) in values {
                    let child_path = format!("{path}/{}", escape_pointer(key));
                    path_bytes = path_bytes
                        .checked_add(child_path.len())
                        .ok_or_else(|| invalid("JSON path byte overflow"))?;
                    if sensitive.contains(&normalized_key(key)) {
                        paths.insert(child_path);
                    } else {
                        stack.push((child_path, child, next_depth));
                    }
                }
            }
            Value::Array(values) => {
                for (index, child) in values.iter().enumerate() {
                    let child_path = format!("{path}/{index}");
                    path_bytes = path_bytes
                        .checked_add(child_path.len())
                        .ok_or_else(|| invalid("JSON path byte overflow"))?;
                    stack.push((child_path, child, next_depth));
                }
            }
            _ => {}
        }
        if nodes.saturating_add(stack.len()) > MAX_JSON_NODES
            || paths.len() > MAX_PATH_MATCHES
            || path_bytes > MAX_PATH_BYTES
        {
            return Err(invalid(
                "provider JSON traversal exceeds its node or path budget",
            ));
        }
    }
    if stack.is_empty() {
        Ok(paths)
    } else {
        Err(invalid("provider JSON node cap reached"))
    }
}

fn remove_pointer(root: &mut Value, pointer: &str) -> Result<(), AwsScanError> {
    let (parent, key) = pointer
        .rsplit_once('/')
        .ok_or_else(|| invalid("redaction path has no parent"))?;
    let key = key.replace("~1", "/").replace("~0", "~");
    match root.pointer_mut(parent) {
        Some(Value::Object(values)) => {
            values.remove(&key);
        }
        Some(Value::Array(values)) => {
            let index = key
                .parse::<usize>()
                .map_err(|_| invalid("redaction array index is invalid"))?;
            if let Some(value) = values.get_mut(index) {
                *value = Value::Null;
            }
        }
        _ => {}
    }
    Ok(())
}

fn sanitized_properties(
    descriptor: &ResourceType,
    raw: Option<&str>,
) -> Result<(Value, Vec<String>), AwsScanError> {
    let raw = raw.unwrap_or("{}");
    if raw.len() > MAX_PROVIDER_BYTES {
        return Err(invalid("Cloud Control Properties exceeds 262144 bytes"));
    }
    let mut value: Value = serde_json::from_str(raw)
        .map_err(|error| invalid(format!("invalid provider JSON: {error}")))?;
    if !value.is_object() {
        return Err(invalid("provider Properties must be a JSON object"));
    }
    let mut redactions = common_redactions(&value, &descriptor.sensitive_names)?;
    for pattern in descriptor
        .write_only_properties
        .iter()
        .chain(&descriptor.sensitive_properties)
    {
        redactions.extend(expand_pointer(&value, pattern)?);
        if redactions.len() > MAX_PATH_MATCHES {
            return Err(invalid("redacted-path count exceeds its bound"));
        }
    }
    let bytes = redactions.iter().try_fold(0_usize, |total, value| {
        total
            .checked_add(value.len())
            .ok_or_else(|| invalid("redacted-path byte overflow"))
    })?;
    if bytes > MAX_PROVIDER_BYTES {
        return Err(invalid("redacted-path metadata exceeds its byte bound"));
    }
    for pointer in &redactions {
        remove_pointer(&mut value, pointer)?;
    }
    Ok((value, redactions.into_iter().collect()))
}

fn arn_parts(value: &str) -> Option<(&str, &str)> {
    let mut parts = value.splitn(6, ':');
    if parts.next()? != "arn" || parts.next()?.is_empty() || parts.next()?.is_empty() {
        return None;
    }
    let region = parts.next()?;
    let account = parts.next()?;
    if parts.next()?.is_empty()
        || value.len() > inventory_core::MAX_IDENTIFIER_BYTES
        || value.contains('\0')
    {
        return None;
    }
    Some((region, account))
}

fn expand_arn(ctx: &Context, template: &str, id: &str) -> Result<String, AwsScanError> {
    if id.is_empty() || id.len() > MAX_IDENTIFIER_BYTES || id.contains(['\0', '|']) {
        return Err(invalid(
            "resource identity template requires one nonempty identifier",
        ));
    }
    let arn = template
        .replace("{partition}", &ctx.partition)
        .replace("{region}", &ctx.region)
        .replace("{account}", &ctx.account)
        .replace("{identifier}", id);
    if arn_parts(&arn).is_none() || arn.contains(['{', '}']) {
        return Err(invalid("invalid identity ARN template"));
    }
    Ok(arn)
}

fn resource_identity(
    ctx: &Context,
    descriptor: &ResourceType,
    identifier: &str,
    properties: &Value,
) -> Result<(String, &'static str), AwsScanError> {
    for pointer in &descriptor.arn_pointers {
        if let Some(arn) = properties.pointer(pointer).and_then(Value::as_str) {
            if arn_parts(arn).is_none() {
                return Err(invalid("provider self-ARN property is malformed"));
            }
            return Ok((arn.to_owned(), "returned_resource_arn"));
        }
    }
    if descriptor.identifier_is_arn && arn_parts(identifier).is_some() {
        return Ok((identifier.to_owned(), "listed_identifier_arn"));
    }
    if let Some(template) = &descriptor.arn_template {
        let property_id = descriptor
            .arn_identifier_pointer
            .as_deref()
            .and_then(|pointer| properties.pointer(pointer))
            .and_then(Value::as_str);
        let simple_identifier = (descriptor.primary_identifier.len() == 1
            && !identifier.contains('|'))
        .then_some(identifier);
        if let Some(id) = property_id.or(simple_identifier) {
            return Ok((
                expand_arn(ctx, template, id)?,
                if template.contains("{account}") {
                    "canonical_template_assumed_scanning_account"
                } else {
                    "canonical_template_without_account_component"
                },
            ));
        }
    }
    Ok((
        format!(
            "awscc:{}:{}:{}:{}:{:x}",
            ctx.partition,
            ctx.region,
            ctx.account,
            descriptor.type_name,
            Sha256::digest(identifier.as_bytes())
        ),
        "namespaced_provider_identifier",
    ))
}

fn tags(properties: &Value) -> Result<BTreeMap<String, String>, AwsScanError> {
    let mut result = BTreeMap::new();
    match properties.get("Tags") {
        Some(Value::Object(values)) => {
            if values.len() > 100 {
                return Err(invalid("provider tag count exceeds 100"));
            }
            for (key, value) in values {
                if let Some(value) = value.as_str() {
                    result.insert(key.clone(), value.to_owned());
                }
            }
        }
        Some(Value::Array(values)) => {
            if values.len() > 100 {
                return Err(invalid("provider tag count exceeds 100"));
            }
            for value in values {
                if let (Some(key), Some(value)) = (
                    value.get("Key").and_then(Value::as_str),
                    value.get("Value").and_then(Value::as_str),
                ) {
                    result.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        _ => {}
    }
    Ok(result)
}

fn map_description(
    ctx: &Context,
    descriptor: &ResourceType,
    identifier: &str,
    description: &ResourceDescription,
    complete: bool,
) -> Result<PageBody, AwsScanError> {
    let (provider, redacted) = sanitized_properties(descriptor, description.properties())?;
    let (id, identity_resolution) = resource_identity(ctx, descriptor, identifier, &provider)?;
    let (region, account) = arn_parts(&id).unwrap_or((&ctx.region, &ctx.account));
    let name = descriptor
        .name_pointers
        .iter()
        .find_map(|pointer| provider.pointer(pointer).and_then(Value::as_str))
        .map(str::to_owned);
    let mut resource = Resource {
        account_id: if account.is_empty() {
            ctx.account.clone()
        } else {
            account.to_owned()
        },
        region: if region.is_empty() {
            ctx.region.clone()
        } else {
            region.to_owned()
        },
        id,
        kind: descriptor.kind.clone(),
        name,
        tags: tags(&provider)?,
        source: Source::CloudControl,
        observed_at: now_millis()?,
        properties: json!({"cloudformation_type":descriptor.type_name,"resource_identifier":identifier,
            "provider_properties":provider,"identity_resolution":identity_resolution,"request_region":ctx.region,
            "schema_sha256":descriptor.schema_sha256,"redacted_paths":redacted,"details_available":complete,
            "observation_detail":if complete {"get_resource"} else {"list_only"}}),
    };
    add_native_aliases(descriptor, &mut resource)?;
    resource.validate()?;
    let edges = if complete {
        map_relationships(ctx, descriptor, &resource)?
    } else {
        Vec::new()
    };
    Ok(PageBody {
        resources: vec![resource],
        edges,
        listed_count: 1,
        next_token: None,
    })
}

fn add_native_aliases(
    descriptor: &ResourceType,
    resource: &mut Resource,
) -> Result<(), AwsScanError> {
    let aliases: &[(&str, &str)] = match descriptor.type_name.as_str() {
        "AWS::EC2::Instance" => &[
            ("instance_id", "InstanceId"),
            ("vpc_id", "VpcId"),
            ("subnet_id", "SubnetId"),
            ("instance_type", "InstanceType"),
            ("image_id", "ImageId"),
            ("private_ip_address", "PrivateIp"),
            ("public_ip_address", "PublicIp"),
        ],
        "AWS::EC2::VPC" => &[("vpc_id", "VpcId"), ("cidr_block", "CidrBlock")],
        "AWS::EC2::Subnet" => &[
            ("subnet_id", "SubnetId"),
            ("vpc_id", "VpcId"),
            ("cidr_block", "CidrBlock"),
            ("availability_zone", "AvailabilityZone"),
        ],
        "AWS::EC2::SecurityGroup" => &[
            ("group_id", "GroupId"),
            ("group_name", "GroupName"),
            ("vpc_id", "VpcId"),
            ("description", "GroupDescription"),
        ],
        "AWS::EC2::RouteTable" => &[("route_table_id", "RouteTableId"), ("vpc_id", "VpcId")],
        "AWS::EC2::NetworkAcl" => &[("network_acl_id", "Id"), ("vpc_id", "VpcId")],
        "AWS::EC2::NatGateway" => &[
            ("nat_gateway_id", "NatGatewayId"),
            ("subnet_id", "SubnetId"),
        ],
        "AWS::EC2::VPCEndpoint" => &[
            ("vpc_endpoint_id", "Id"),
            ("vpc_id", "VpcId"),
            ("vpc_endpoint_type", "VpcEndpointType"),
        ],
        "AWS::DynamoDB::Table" => &[("table_name", "TableName")],
        "AWS::Lambda::Function" => &[
            ("function_name", "FunctionName"),
            ("runtime", "Runtime"),
            ("handler", "Handler"),
            ("role", "Role"),
            ("memory_size", "MemorySize"),
            ("timeout", "Timeout"),
        ],
        "AWS::ECS::Cluster" => &[("cluster_name", "ClusterName"), ("cluster_arn", "Arn")],
        "AWS::ECS::Service" => &[
            ("service_name", "Name"),
            ("service_arn", "ServiceArn"),
            ("task_definition", "TaskDefinition"),
            ("desired_count", "DesiredCount"),
        ],
        "AWS::ECS::TaskDefinition" => &[
            ("task_definition_arn", "TaskDefinitionArn"),
            ("family", "Family"),
            ("network_mode", "NetworkMode"),
            ("task_role_arn", "TaskRoleArn"),
            ("execution_role_arn", "ExecutionRoleArn"),
        ],
        "AWS::EKS::Cluster" => &[
            ("name", "Name"),
            ("arn", "Arn"),
            ("version", "Version"),
            ("endpoint", "Endpoint"),
        ],
        "AWS::ElasticLoadBalancingV2::LoadBalancer" => &[
            ("type", "Type"),
            ("load_balancer_arn", "LoadBalancerArn"),
            ("load_balancer_name", "LoadBalancerName"),
            ("scheme", "Scheme"),
            ("dns_name", "DNSName"),
        ],
        "AWS::ElasticLoadBalancingV2::TargetGroup" => &[
            ("target_group_arn", "TargetGroupArn"),
            ("target_group_name", "TargetGroupName"),
            ("vpc_id", "VpcId"),
            ("port", "Port"),
            ("protocol", "Protocol"),
            ("target_type", "TargetType"),
        ],
        _ => &[],
    };
    let properties = resource
        .properties
        .as_object_mut()
        .ok_or_else(|| invalid("resource properties must be an object"))?;
    for (alias, key) in aliases {
        if let Some(value) = properties
            .get("provider_properties")
            .and_then(|provider| provider.get(*key))
            .filter(|value| value.is_string() || value.is_number() || value.is_boolean())
            .cloned()
        {
            properties.insert((*alias).to_owned(), value);
        }
    }
    Ok(())
}

fn reference_arn_matches(rule: &Relationship, value: &str) -> bool {
    if arn_parts(value).is_none() || value.chars().any(char::is_control) {
        return false;
    }
    let mut parts = value.splitn(6, ':');
    let (Some(_), Some(partition), Some(service), Some(region), Some(account), Some(resource)) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return false;
    };
    let component = |value: &str| {
        value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    };
    if !component(partition)
        || !component(service)
        || !component(region)
        || (!account.is_empty()
            && !(account.len() == 12 && account.bytes().all(|byte| byte.is_ascii_digit()))
            && !(service == "iam" && account == "aws" && resource.starts_with("policy/")))
    {
        return false;
    }
    // CloudWatch Logs returns this suffix for a concrete log group's ARN.
    // Other wildcard/resource-pattern expressions are not concrete targets.
    let concrete_log_group = service == "logs"
        && resource.strip_suffix(":*").is_some_and(|group| {
            group.starts_with("log-group:") && !group.contains(['*', '?', '{', '}'])
        });
    if value.contains(['?', '{', '}']) || (value.contains('*') && !concrete_log_group) {
        return false;
    }
    rule.arn_service
        .as_deref()
        .is_none_or(|expected| service == expected)
        && rule.arn_resource_prefix.as_deref().is_none_or(|prefix| {
            resource
                .strip_prefix(prefix)
                .is_some_and(|identifier| !identifier.is_empty())
        })
}

fn reference_identifier_matches(rule: &Relationship, value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
        return false;
    }
    match rule.identifier_prefix.as_deref() {
        Some(prefix) => value.strip_prefix(prefix).is_some_and(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        }),
        None => !value.contains(['\0', '|']),
    }
}

fn map_relationships(
    ctx: &Context,
    descriptor: &ResourceType,
    resource: &Resource,
) -> Result<Vec<Edge>, AwsScanError> {
    let provider = resource
        .properties
        .get("provider_properties")
        .ok_or_else(|| invalid("missing sanitized provider properties"))?;
    let mut merged: BTreeMap<(String, String), Edge> = BTreeMap::new();
    for rule in &descriptor.relationships {
        for path in expand_pointer(provider, &rule.pointer)? {
            let Some(value) = provider
                .pointer(&path)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let (target, owner_resolution) = if rule.target_format == "arn" {
                if !reference_arn_matches(rule, value) {
                    continue;
                }
                (value.to_owned(), "returned_arn")
            } else {
                if !reference_identifier_matches(rule, value) {
                    continue;
                }
                let template = rule
                    .arn_template
                    .as_deref()
                    .ok_or_else(|| invalid("relationship ARN template absent"))?;
                (
                    expand_arn(ctx, template, value)?,
                    if template.contains("{account}") {
                        "assumed_scanning_account"
                    } else {
                        "arn_has_no_account_component"
                    },
                )
            };
            if target == resource.id {
                continue;
            }
            let key = (target.clone(), rule.relation.clone());
            if let Some(edge) = merged.get_mut(&key) {
                let paths = edge
                    .properties
                    .get_mut("property_paths")
                    .and_then(Value::as_array_mut)
                    .ok_or_else(|| invalid("edge path evidence is malformed"))?;
                if !paths.iter().any(|value| value.as_str() == Some(&path)) {
                    paths.push(Value::String(path));
                }
            } else {
                if merged.len() >= MAX_RESOURCE_EDGES {
                    return Err(invalid("resource relationship bound exceeded"));
                }
                let edge = Edge {
                    from_id: resource.id.clone(),
                    to_id: target,
                    relation: rule.relation.clone(),
                    source: Source::CloudControl,
                    observed_at: resource.observed_at,
                    properties: json!({"target_resource_id":value,"target_kind":rule.target_kind,"owner_resolution":owner_resolution,
                        "cloudformation_type":descriptor.type_name,"property_paths":[path],"evidence":rule.evidence,
                        "observation_detail":resource.properties.get("observation_detail")}),
                };
                edge.validate()?;
                merged.insert(key, edge);
            }
        }
    }
    Ok(merged.into_values().collect())
}

#[cfg(test)]
mod architecture_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::mpsc;

    type TestError = Box<dyn std::error::Error>;
    type TestResult = Result<(), TestError>;

    fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
        let (sender, receiver) = mpsc::channel(64);
        Ok((
            Context {
                config: aws_types::SdkConfig::builder().build(),
                executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
                sender,
                account: "111111111111".into(),
                partition: "aws".into(),
                region: "us-east-1".into(),
                max_pages,
                page_size: 100,
            },
            receiver,
        ))
    }

    fn descriptor(name: &str) -> Result<ResourceType, TestError> {
        catalog()?
            .into_iter()
            .find(|descriptor| descriptor.type_name == name)
            .ok_or_else(|| format!("missing fixture type {name}").into())
    }

    fn description(id: &str, properties: Value) -> ResourceDescription {
        ResourceDescription::builder()
            .identifier(id)
            .properties(properties.to_string())
            .build()
    }

    fn get(description: ResourceDescription) -> GetResourceOutput {
        GetResourceOutput::builder()
            .resource_description(description)
            .build()
    }

    fn first(body: &PageBody) -> Result<&Resource, TestError> {
        body.resources
            .first()
            .ok_or_else(|| "missing resource".into())
    }

    fn drain(
        receiver: &mut mpsc::Receiver<CollectionEvent>,
    ) -> Result<Vec<CollectionEvent>, TestError> {
        let mut events = Vec::new();
        for _ in 0..64 {
            match receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                    return Ok(events);
                }
            }
        }
        Err("fixture event capacity exceeded".into())
    }

    #[test]
    fn catalog_has_397_reviewed_regional_read_list_types_and_safe_columns() -> TestResult {
        let catalog = root_catalog()?;
        assert_eq!(catalog.len(), 397);
        let names: BTreeSet<&str> = catalog
            .iter()
            .map(|entry| entry.type_name.as_str())
            .collect();
        assert_eq!(names.len(), 397);
        let services: BTreeSet<&str> = catalog.iter().map(|entry| entry.service.as_str()).collect();
        assert_eq!(services.len(), 76);
        for absent in [
            "AWS::ElasticLoadBalancingV2::Listener",
            "AWS::ElasticLoadBalancingV2::ListenerRule",
            "AWS::RDS::DBProxyTargetGroup",
            "AWS::EKS::FargateProfile",
            "AWS::IAM::Role",
            "AWS::S3::Bucket",
        ] {
            assert!(!names.contains(absent));
        }
        let reserved = [
            "id",
            "arn",
            "name",
            "region",
            "account_id",
            "kind",
            "source",
            "source_priority",
            "observed_at",
            "properties",
            "tags",
            "resource_json",
            "scan_id",
            "resource_identifier",
            "provider_properties",
            "cloudformation_type",
        ];
        for entry in &catalog {
            assert!(entry.read_supported && entry.list_supported && !entry.list_requires_model);
            assert_eq!(entry.scope, "regional");
            assert!(entry.columns.len() <= 200);
            assert!(
                entry
                    .columns
                    .iter()
                    .all(|column| !reserved.contains(&column.name.as_str()))
            );
        }
        assert_eq!(
            descriptor("AWS::Logs::LogGroup")?.list_model_validation,
            "empty_object_accepted"
        );
        Ok(())
    }

    #[test]
    fn selection_rejects_unknown_types_and_deduplicates_exact_names() -> TestResult {
        assert_eq!(validate_selection(&[])?.len(), 418);
        let selection =
            validate_selection(&["AWS::EC2::Instance".into(), "AWS::EC2::Instance".into()])?;
        assert_eq!(selection.len(), 1);
        assert!(validate_selection(&["AWS::IAM::Role".into()]).is_err());
        assert!(validate_selection(&vec!["AWS::EC2::Instance".into(); 419]).is_err());
        Ok(())
    }

    #[test]
    fn canonical_identity_matches_native_and_returned_owner_is_preserved() -> TestResult {
        let (ctx, _events) = context(3)?;
        let ec2 = descriptor("AWS::EC2::Instance")?;
        let body = map_get(
            &ctx,
            &ec2,
            "i-abc",
            &get(description(
                "i-abc",
                json!({"InstanceId":"i-abc","VpcId":"vpc-a","SubnetId":"subnet-a","InstanceType":"t3.micro","UserData":"NEVER_STORE"}),
            )),
        )?;
        let resource = first(&body)?;
        assert_eq!(
            resource.id,
            "arn:aws:ec2:us-east-1:111111111111:instance/i-abc"
        );
        assert_eq!(resource.source, Source::CloudControl);
        assert_eq!(
            resource.properties.get("instance_type"),
            Some(&json!("t3.micro"))
        );
        assert_eq!(body.edges.len(), 2);
        assert!(!serde_json::to_string(resource)?.contains("NEVER_STORE"));
        let lambda = descriptor("AWS::Lambda::Function")?;
        let arn = "arn:aws:lambda:eu-west-1:222222222222:function:handler";
        let body = map_get(
            &ctx,
            &lambda,
            "handler",
            &get(description(
                "handler",
                json!({"FunctionName":"handler","Arn":arn}),
            )),
        )?;
        let resource = first(&body)?;
        assert_eq!(resource.id, arn);
        assert_eq!(resource.account_id, "222222222222");
        assert_eq!(resource.region, "eu-west-1");
        Ok(())
    }

    #[test]
    fn composite_eip_identity_uses_allocation_property_and_exact_identifier_is_retained()
    -> TestResult {
        let (ctx, _events) = context(3)?;
        let kind = descriptor("AWS::EC2::EIP")?;
        let id = "203.0.113.5|eipalloc-abcd";
        let body = map_get(
            &ctx,
            &kind,
            id,
            &get(description(
                id,
                json!({"PublicIp":"203.0.113.5","AllocationId":"eipalloc-abcd"}),
            )),
        )?;
        let resource = first(&body)?;
        assert_eq!(
            resource.id,
            "arn:aws:ec2:us-east-1:111111111111:elastic-ip/eipalloc-abcd"
        );
        assert_eq!(
            resource.properties.get("resource_identifier"),
            Some(&json!(id))
        );
        let empty = description(id, json!({}));
        let first_fallback = map_description(&ctx, &kind, id, &empty, false)?;
        let second_fallback = map_description(&ctx, &kind, id, &empty, false)?;
        assert_eq!(first(&first_fallback)?.id, first(&second_fallback)?.id);
        assert!(
            first(&first_fallback)?
                .id
                .starts_with("awscc:aws:us-east-1:111111111111:AWS::EC2::EIP:")
        );
        assert!(!first(&first_fallback)?.id.contains("203.0.113.5"));
        Ok(())
    }

    #[test]
    fn fallback_identity_namespaces_identical_provider_ids_by_type() -> TestResult {
        let (ctx, _events) = context(3)?;
        let first_kind = descriptor("AWS::AppConfig::Application")?;
        let second_kind = descriptor("AWS::AppConfig::DeploymentStrategy")?;
        let value = description("same-id", json!({"Name":"same-name"}));
        let left = map_description(&ctx, &first_kind, "same-id", &value, false)?;
        let right = map_description(&ctx, &second_kind, "same-id", &value, false)?;
        assert_ne!(first(&left)?.id, first(&right)?.id);
        assert!(left.edges.is_empty() && right.edges.is_empty());
        Ok(())
    }

    #[test]
    fn self_arn_is_used_instead_of_a_borrowed_role_or_secret_arn() -> TestResult {
        let (ctx, _events) = context(3)?;
        let kind = descriptor("AWS::Batch::JobDefinition")?;
        let arn = "arn:aws:batch:us-east-1:111111111111:job-definition/build:7";
        let body = map_description(
            &ctx,
            &kind,
            "build:7",
            &description(
                "build:7",
                json!({"JobDefinitionArn":arn,"RoleArn":"arn:aws:iam::111111111111:role/build"}),
            ),
            true,
        )?;
        assert_eq!(first(&body)?.id, arn);
        let redshift = descriptor("AWS::Redshift::Cluster")?;
        assert!(!redshift.arn_pointers.iter().any(|value| value == "/ClusterNamespaceArn" || value == "/MasterPasswordSecretArn"));
        Ok(())
    }

    #[test]
    fn wildcard_write_only_fields_and_common_nested_credentials_are_removed() -> TestResult {
        let kind = descriptor("AWS::EC2::SecurityGroup")?;
        let (safe, paths) = sanitized_properties(&kind, Some(&json!({"GroupId":"sg-a","SecurityGroupIngress":[{"SourceSecurityGroupName":"HIDDEN","IpProtocol":"tcp"},{"SourceSecurityGroupName":"HIDDEN_TWO","FromPort":443}],"Metadata":{"client_secret":"HIDDEN_THREE","Name":"visible"}}).to_string()))?;
        assert!(
            safe.pointer("/SecurityGroupIngress/0/SourceSecurityGroupName")
                .is_none()
        );
        assert_eq!(
            safe.pointer("/SecurityGroupIngress/0/IpProtocol"),
            Some(&json!("tcp"))
        );
        assert!(
            safe.pointer("/SecurityGroupIngress/1/SourceSecurityGroupName")
                .is_none()
        );
        assert!(safe.pointer("/Metadata/client_secret").is_none());
        assert_eq!(safe.pointer("/Metadata/Name"), Some(&json!("visible")));
        assert!(paths.contains(&"/SecurityGroupIngress/0/SourceSecurityGroupName".into()));
        assert!(!safe.to_string().contains("HIDDEN"));
        Ok(())
    }

    #[test]
    fn read_visible_environment_commands_parameter_values_and_firehose_tokens_are_redacted()
    -> TestResult {
        for (name, raw) in [
            (
                "AWS::Lambda::Function",
                json!({"Environment":{"Variables":{"TOKEN":"HIDDEN"}},"Code":{"ZipFile":"HIDDEN"},"Role":"arn:aws:iam::111111111111:role/application"}),
            ),
            (
                "AWS::ECS::TaskDefinition",
                json!({"ContainerDefinitions":[{"Name":"app","Environment":[{"Name":"TOKEN","Value":"HIDDEN"}],"Command":["HIDDEN"],"LogConfiguration":{"Options":{"secret":"HIDDEN"}}}]}),
            ),
            (
                "AWS::Batch::JobDefinition",
                json!({"Parameters":{"input":"HIDDEN"},"EksProperties":{"PodProperties":{"Containers":[{"Env":[{"Name":"TOKEN","Value":"HIDDEN"}],"Args":["HIDDEN"]}],"InitContainers":[{"Env":[{"Name":"TOKEN","Value":"HIDDEN"}]}]}},"NodeProperties":{"NodeRangeProperties":[{"EksProperties":{"PodProperties":{"Containers":[{"Env":[{"Value":"HIDDEN"}]}]}}}]}}),
            ),
            (
                "AWS::SSM::Parameter",
                json!({"Name":"/app/config","Value":"HIDDEN"}),
            ),
            (
                "AWS::KinesisFirehose::DeliveryStream",
                json!({"SplunkDestinationConfiguration":{"HECToken":"HIDDEN"},"HttpEndpointDestinationConfiguration":{"EndpointConfiguration":{"AccessKey":"HIDDEN"}}}),
            ),
            (
                "AWS::SNS::Subscription",
                json!({"Protocol":"https","Endpoint":"https://user:HIDDEN@example.test/notify"}),
            ),
            (
                "AWS::Pipes::Pipe",
                json!({"EnrichmentParameters":{"HttpParameters":{"HeaderParameters":{"Authorization":"HIDDEN"},"QueryStringParameters":{"key":"HIDDEN"}}}}),
            ),
            (
                "AWS::RDS::OptionGroup",
                json!({"OptionConfigurations":[{"OptionName":"OEM_AGENT","OptionSettings":[{"Name":"AGENT_REGISTRATION_PASSWORD","Value":"HIDDEN"}]}]}),
            ),
        ] {
            let (safe, redacted) =
                sanitized_properties(&descriptor(name)?, Some(&raw.to_string()))?;
            assert!(
                !safe.to_string().contains("HIDDEN"),
                "sensitive value survived for {name}"
            );
            assert!(!redacted.is_empty());
        }
        Ok(())
    }

    #[test]
    fn relationships_deduplicate_paths_and_preserve_returned_arn_ownership() -> TestResult {
        let (ctx, _events) = context(3)?;
        let kind = descriptor("AWS::ElasticLoadBalancingV2::LoadBalancer")?;
        let arn = "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/demo/123";
        let body = map_description(
            &ctx,
            &kind,
            arn,
            &description(
                arn,
                json!({"LoadBalancerArn":arn,"Type":"application","Subnets":["subnet-one","subnet-one"],"SubnetMappings":[{"SubnetId":"subnet-one"}]}),
            ),
            true,
        )?;
        assert_eq!(body.edges.len(), 1);
        let edge = body.edges.first().ok_or("missing edge")?;
        assert_eq!(
            edge.to_id,
            "arn:aws:ec2:us-east-1:111111111111:subnet/subnet-one"
        );
        assert_eq!(
            edge.properties.get("owner_resolution"),
            Some(&json!("assumed_scanning_account"))
        );
        assert_eq!(
            edge.properties
                .get("property_paths")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(3)
        );
        let lambda = descriptor("AWS::Lambda::Function")?;
        let body = map_description(
            &ctx,
            &lambda,
            "demo",
            &description(
                "demo",
                json!({"FunctionName":"demo","Role":"arn:aws:iam::222222222222:role/cross-account"}),
            ),
            true,
        )?;
        assert_eq!(
            body.edges.first().ok_or("missing role edge")?.to_id,
            "arn:aws:iam::222222222222:role/cross-account"
        );
        Ok(())
    }

    #[test]
    fn malformed_get_responses_do_not_become_empty_successes() -> TestResult {
        let (ctx, _events) = context(3)?;
        let kind = descriptor("AWS::EC2::Instance")?;
        for output in [
            GetResourceOutput::builder().build(),
            get(ResourceDescription::builder().identifier("i-one").build()),
            get(description("i-other", json!({}))),
            GetResourceOutput::builder()
                .type_name("AWS::EC2::VPC")
                .resource_description(description("i-one", json!({})))
                .build(),
        ] {
            assert!(map_get(&ctx, &kind, "i-one", &output).is_err());
        }
        assert!(sanitized_properties(&kind, Some("[]")).is_err());
        assert!(sanitized_properties(&kind, Some("not-json")).is_err());
        assert!(
            sanitized_properties(
                &kind,
                Some(&" ".repeat(MAX_PROVIDER_BYTES.saturating_add(1)))
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn input_node_depth_and_pointer_expansion_caps_fail_explicitly() -> TestResult {
        let kind = descriptor("AWS::AppConfig::Application")?;
        let many = json!({"Items":vec![0; MAX_JSON_NODES]});
        assert!(sanitized_properties(&kind, Some(&many.to_string())).is_err());
        let mut nested = json!("leaf");
        for _ in 0..MAX_JSON_DEPTH.saturating_add(2) {
            nested = json!({"Nested":nested});
        }
        assert!(sanitized_properties(&kind, Some(&nested.to_string())).is_err());
        let many = json!({"Items":vec![json!({"Id":"one"});MAX_PATH_MATCHES.saturating_add(1)]});
        assert!(expand_pointer(&many, "/Items/*/Id").is_err());
        Ok(())
    }

    #[test]
    fn list_identity_ledger_rejects_replays_before_enrichment_and_does_not_consume_failed_budget()
    -> TestResult {
        let kind = descriptor("AWS::EC2::Instance")?;
        let ledger = Mutex::new(IdentityLedger::default());
        let one = ResourceDescription::builder().identifier("i-one").build();
        let duplicate = ListResourcesOutput::builder()
            .resource_descriptions(one.clone())
            .resource_descriptions(one.clone())
            .build();
        assert!(remember_list_page(&kind, &duplicate, &ledger, MAX_CHILDREN).is_err());
        let first = ListResourcesOutput::builder()
            .resource_descriptions(one)
            .build();
        assert!(remember_list_page(&kind, &first, &ledger, 0).is_err());
        remember_list_page(&kind, &first, &ledger, MAX_CHILDREN)?;
        assert!(remember_list_page(&kind, &first, &ledger, MAX_CHILDREN).is_err());
        let invalid = ListResourcesOutput::builder()
            .resource_descriptions(ResourceDescription::builder().build())
            .build();
        assert!(remember_list_page(&kind, &invalid, &ledger, MAX_CHILDREN).is_err());
        assert!(
            remember_list_page(
                &kind,
                &ListResourcesOutput::builder().next_token("").build(),
                &ledger,
                MAX_CHILDREN
            )
            .is_err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn empty_terminal_list_is_complete_and_get_is_not_called() -> TestResult {
        let (ctx, mut events) = context(3)?;
        let kind = descriptor("AWS::EC2::Instance")?;
        let calls = AtomicUsize::new(0);
        collect_type_with(
            &ctx,
            &kind,
            |_| async { Ok(ListResourcesOutput::builder().build()) },
            |_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(ApiError::Permanent("unexpected read".into()))
            },
        )
        .await?;
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let collected = drain(&mut events)?;
        assert_eq!(collected.len(), 1);
        assert!(collected.iter().all(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" && page.next_token.is_none() && page.resources.is_empty())));
        Ok(())
    }

    #[tokio::test]
    async fn absent_list_properties_still_enriches_with_the_exact_composite_identifier()
    -> TestResult {
        let (ctx, mut events) = context(3)?;
        let kind = descriptor("AWS::EC2::EIP")?;
        let id = "203.0.113.5|eipalloc-abcd";
        collect_type_with(
            &ctx,
            &kind,
            |_| async {
                Ok(ListResourcesOutput::builder()
                    .resource_descriptions(ResourceDescription::builder().identifier(id).build())
                    .build())
            },
            |received| async move {
                assert_eq!(received, id);
                Ok(get(description(
                    &received,
                    json!({"PublicIp":"203.0.113.5","AllocationId":"eipalloc-abcd"}),
                )))
            },
        )
        .await?;
        let collected = drain(&mut events)?;
        assert_eq!(collected.len(), 2);
        assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_get" && page.resources.len() == 1)));
        assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" && page.resources.is_empty() && page.edges.is_empty())));
        Ok(())
    }

    #[tokio::test]
    async fn denied_and_malformed_children_do_not_erase_valid_siblings() -> TestResult {
        let (ctx, mut events) = context(3)?;
        let kind = descriptor("AWS::EC2::Instance")?;
        collect_type_with(
            &ctx,
            &kind,
            |_| async {
                Ok(ListResourcesOutput::builder()
                    .resource_descriptions(
                        ResourceDescription::builder()
                            .identifier("i-malformed")
                            .properties("not-json")
                            .build(),
                    )
                    .resource_descriptions(description(
                        "i-denied",
                        json!({"InstanceId":"i-denied","VpcId":"vpc-one","UserData":"HIDDEN"}),
                    ))
                    .resource_descriptions(
                        ResourceDescription::builder()
                            .identifier("i-success")
                            .build(),
                    )
                    .build())
            },
            |id| async move {
                if id == "i-success" {
                    Ok(get(description(
                        &id,
                        json!({"InstanceId":id,"InstanceType":"t3.micro"}),
                    )))
                } else {
                    Err(ApiError::Denied("synthetic denied GetResource".into()))
                }
            },
        )
        .await?;
        let collected = drain(&mut events)?;
        let resources: Vec<&Resource> = collected
            .iter()
            .filter_map(|event| match event {
                CollectionEvent::Page(page) => Some(page.resources.iter()),
                _ => None,
            })
            .flatten()
            .collect();
        assert_eq!(resources.len(), 2);
        assert!(
            resources
                .iter()
                .any(|resource| resource.properties.get("resource_identifier")
                    == Some(&json!("i-success"))
                    && resource.properties.get("details_available") == Some(&json!(true)))
        );
        assert!(
            resources
                .iter()
                .any(|resource| resource.properties.get("resource_identifier")
                    == Some(&json!("i-denied"))
                    && resource.properties.get("details_available") == Some(&json!(false)))
        );
        assert!(!serde_json::to_string(&collected)?.contains("HIDDEN"));
        assert_eq!(
            collected
                .iter()
                .filter(|event| matches!(event, CollectionEvent::Failure(_)))
                .count(),
            3
        );
        assert!(
            collected
                .iter()
                .filter_map(|event| match event {
                    CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" =>
                        Some(page),
                    _ => None,
                })
                .all(|page| page.resources.is_empty() && page.edges.is_empty())
        );
        let denied_page = collected.iter().position(|event| matches!(event, CollectionEvent::Page(page) if page.resources.iter().any(|resource| resource.properties.get("resource_identifier") == Some(&json!("i-denied"))))).ok_or("missing denied observation")?;
        let denied_scope = match collected.get(denied_page) {
            Some(CollectionEvent::Page(page)) => &page.scope,
            _ => return Err("wrong event".into()),
        };
        assert!(collected.iter().take(denied_page).any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope == *denied_scope)));
        Ok(())
    }

    #[tokio::test]
    async fn empty_page_with_token_continues_and_page_cap_remains_incomplete() -> TestResult {
        let kind = descriptor("AWS::EC2::Instance")?;
        for (max_pages, expected_failure) in [(2, false), (1, true)] {
            let (ctx, mut events) = context(max_pages)?;
            collect_type_with(
                &ctx,
                &kind,
                |token| async move {
                    Ok(if token.is_none() {
                        ListResourcesOutput::builder().next_token("next").build()
                    } else {
                        ListResourcesOutput::builder().build()
                    })
                },
                |_| async { Err(ApiError::Permanent("unexpected read".into())) },
            )
            .await?;
            let collected = drain(&mut events)?;
            assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.page_number == 1 && page.next_token.as_deref() == Some("next"))));
            assert_eq!(
                collected
                    .iter()
                    .any(|event| matches!(event, CollectionEvent::Failure(_))),
                expected_failure
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn repeated_identifier_across_pages_fails_the_scope_without_replaying_child() -> TestResult
    {
        let (ctx, mut events) = context(3)?;
        let kind = descriptor("AWS::EC2::Instance")?;
        let calls = AtomicUsize::new(0);
        collect_type_with(
            &ctx,
            &kind,
            |token| async move {
                Ok(ListResourcesOutput::builder()
                    .resource_descriptions(
                        ResourceDescription::builder().identifier("i-one").build(),
                    )
                    .set_next_token(token.is_none().then(|| "next".to_owned()))
                    .build())
            },
            |id| {
                calls.fetch_add(1, Ordering::SeqCst);
                async move { Ok(get(description(&id, json!({"InstanceId":id})))) }
            },
        )
        .await?;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let collected = drain(&mut events)?;
        assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == "cloud_control_list")));
        assert!(!collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" && page.next_token.is_none())));
        Ok(())
    }

    #[tokio::test]
    async fn unsupported_list_and_closed_consumer_never_claim_empty_success() -> TestResult {
        let kind = descriptor("AWS::EC2::Instance")?;
        let (ctx, mut events) = context(3)?;
        collect_type_with(
            &ctx,
            &kind,
            |_| async { Err(ApiError::Permanent("UnsupportedAction".into())) },
            |_| async { Err(ApiError::Permanent("unexpected read".into())) },
        )
        .await?;
        let collected = drain(&mut events)?;
        assert_eq!(collected.len(), 1);
        assert!(matches!(
            collected.first(),
            Some(CollectionEvent::Failure(_))
        ));
        drop(events);
        let result = collect_type_with(
            &ctx,
            &kind,
            |_| async {
                Ok(ListResourcesOutput::builder()
                    .resource_descriptions(
                        ResourceDescription::builder().identifier("i-one").build(),
                    )
                    .build())
            },
            |id| async move { Ok(get(description(&id, json!({"InstanceId":id})))) },
        )
        .await;
        assert!(matches!(result, Err(AwsScanError::ChannelClosed)));
        Ok(())
    }

    #[tokio::test]
    async fn pre_cancelled_collection_admits_no_types_or_children() -> TestResult {
        let (ctx, mut events) = context(3)?;
        let kind = descriptor("AWS::EC2::Instance")?;
        ctx.executor.cancel();
        let calls = AtomicUsize::new(0);
        let result = collect_type_with(
            &ctx,
            &kind,
            |_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(ListResourcesOutput::builder().build())
            },
            |_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(ApiError::Permanent("unexpected request".into()))
            },
        )
        .await;
        assert!(matches!(
            result,
            Err(AwsScanError::Api(ApiError::Cancelled))
        ));
        assert!(matches!(
            collect(&ctx, &[kind]).await,
            Err(AwsScanError::Api(ApiError::Cancelled))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(drain(&mut events)?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn mid_page_cancellation_stops_queued_children_without_list_fallbacks() -> TestResult {
        let (ctx, mut events) = context(3)?;
        let kind = descriptor("AWS::EC2::Instance")?;
        let calls = AtomicUsize::new(0);
        let result = collect_type_with(
            &ctx,
            &kind,
            |_| async {
                Ok(ListResourcesOutput::builder()
                    .set_resource_descriptions(Some(
                        (0..30)
                            .map(|index| {
                                description(
                                    &format!("i-{index}"),
                                    json!({"InstanceType":"t3.micro"}),
                                )
                            })
                            .collect(),
                    ))
                    .build())
            },
            |_| async {
                calls.fetch_add(1, Ordering::SeqCst);
                ctx.executor.cancel();
                Err(ApiError::Cancelled)
            },
        )
        .await;
        assert!(matches!(
            result,
            Err(AwsScanError::Api(ApiError::Cancelled))
        ));
        assert!(calls.load(Ordering::SeqCst) <= READ_CONCURRENCY);
        let collected = drain(&mut events)?;
        assert_eq!(collected.len(), 1);
        assert!(
            matches!(collected.first(), Some(CollectionEvent::Failure(failure)) if failure.category == "cancelled")
        );
        Ok(())
    }
}
