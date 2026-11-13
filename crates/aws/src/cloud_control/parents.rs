//! Reviewed one-level parent discovery with explicit incomplete prerequisites.
//!
//! Parents are streamed once per root type. Only bounded models made from
//! sanitized, successful reads drive child lists; no full parent inventory is
//! retained. A child read must independently agree with its requested parent.

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

pub(super) const PARENT_TYPE_COUNT: usize = 21;
const PARENT_CATALOG_JSON: &str =
    include_str!("../../../../catalog/cloudcontrol-parent-resources.json");
const CHILD_TYPE_CONCURRENCY: usize = 2;
const MAX_MODELS_PER_ROOT: usize = 10_000;
const MAX_MODEL_BYTES: usize = 8_192;
const MAX_BINDINGS: usize = 4;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Binding {
    source_pointer: String,
    model_property: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParentPlan {
    pub type_name: String,
    pub parent_type: String,
    bindings: Vec<Binding>,
    relation: String,
    evidence: String,
}

#[derive(Deserialize)]
struct ParentCatalog {
    schema_version: u32,
    evidence: CatalogEvidence,
    resource_types: Vec<ResourceType>,
    parent_plans: Vec<ParentPlan>,
}

fn read_catalog() -> Result<ParentCatalog, AwsScanError> {
    let catalog: ParentCatalog = serde_json::from_str(PARENT_CATALOG_JSON)
        .map_err(|error| invalid(format!("embedded parent catalog: {error}")))?;
    if catalog.schema_version != 1
        || catalog.resource_types.len() != PARENT_TYPE_COUNT
        || catalog.parent_plans.len() != PARENT_TYPE_COUNT
        || catalog.evidence.global_sensitive_property_names.is_empty()
        || catalog.evidence.global_sensitive_property_names.len() > 128
    {
        return Err(invalid(
            "invalid parent catalog version, count or redaction policy",
        ));
    }
    Ok(catalog)
}

pub(super) fn catalog() -> Result<Vec<ResourceType>, AwsScanError> {
    let mut catalog = read_catalog()?;
    let sensitive = Arc::new(catalog.evidence.global_sensitive_property_names);
    for descriptor in &mut catalog.resource_types {
        validate_descriptor(descriptor, true)?;
        if descriptor.list_model_validation != "reviewed_parent_model" {
            return Err(invalid(
                "parent descriptor lacks reviewed list-model evidence",
            ));
        }
        descriptor.sensitive_names = Arc::clone(&sensitive);
    }
    Ok(catalog.resource_types)
}

pub(super) fn plans() -> Result<Vec<ParentPlan>, AwsScanError> {
    Ok(read_catalog()?.parent_plans)
}

pub(super) fn validate_catalog(types: &[ResourceType]) -> Result<(), AwsScanError> {
    let mut names = BTreeSet::new();
    let mut views = BTreeSet::new();
    if types.len() != TYPE_COUNT {
        return Err(invalid("combined catalog has an unexpected type count"));
    }
    for descriptor in types {
        if !names.insert(descriptor.type_name.as_str())
            || !views.insert(descriptor.view_name.as_str())
        {
            return Err(invalid("combined catalog repeats a type or view"));
        }
    }
    let mut children = BTreeSet::new();
    for plan in plans()? {
        let child = find_type(types, &plan.type_name)?;
        let parent = find_type(types, &plan.parent_type)?;
        if !children.insert(plan.type_name.clone())
            || !child.list_requires_model
            || parent.list_requires_model
            || plan.bindings.is_empty()
            || plan.bindings.len() > MAX_BINDINGS
            || plan.relation != "CHILD_OF"
            || plan.evidence != "reviewed_parent_resource_model_and_matching_read_properties"
        {
            return Err(invalid("invalid or non-root parent dependency"));
        }
        validate_bindings(&plan, parent, child)?;
    }
    if children.len() != PARENT_TYPE_COUNT {
        return Err(invalid("parent catalog has missing plans"));
    }
    Ok(())
}

fn validate_bindings(
    plan: &ParentPlan,
    parent: &ResourceType,
    child: &ResourceType,
) -> Result<(), AwsScanError> {
    let mut properties = BTreeSet::new();
    for binding in &plan.bindings {
        validate_pointer(&binding.source_pointer)?;
        let target = format!("/{}", binding.model_property);
        if binding.model_property.is_empty()
            || binding.model_property.len() > 128
            || !binding
                .model_property
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric())
            || !properties.insert(&binding.model_property)
            || !parent.columns.iter().any(|column| {
                column.pointer == binding.source_pointer && column.json_type == "string"
            })
            || !child
                .columns
                .iter()
                .any(|column| column.pointer == target && column.json_type == "string")
        {
            return Err(invalid("parent binding is not a readable string property"));
        }
    }
    Ok(())
}

fn find_type<'a>(types: &'a [ResourceType], name: &str) -> Result<&'a ResourceType, AwsScanError> {
    types
        .iter()
        .find(|descriptor| descriptor.type_name == name)
        .ok_or_else(|| invalid(format!("missing selected prerequisite type {name}")))
}

pub(super) struct ChildType<'a> {
    descriptor: &'a ResourceType,
    plan: ParentPlan,
}

pub(super) struct RootGroup<'a> {
    descriptor: &'a ResourceType,
    children: Vec<ChildType<'a>>,
}

pub(super) fn groups(types: &[ResourceType]) -> Result<Vec<RootGroup<'_>>, AwsScanError> {
    let plans = plans()?;
    let mut groups: Vec<RootGroup<'_>> = types
        .iter()
        .filter(|descriptor| !descriptor.list_requires_model)
        .map(|descriptor| RootGroup {
            descriptor,
            children: Vec::new(),
        })
        .collect();
    for descriptor in types
        .iter()
        .filter(|descriptor| descriptor.list_requires_model)
    {
        let plan = plans
            .iter()
            .find(|plan| plan.type_name == descriptor.type_name)
            .ok_or_else(|| invalid("selected parent-dependent type lacks a plan"))?;
        let group = groups
            .iter_mut()
            .find(|group| group.descriptor.type_name == plan.parent_type)
            .ok_or_else(|| invalid("parent-dependent selection lacks its prerequisite"))?;
        group.children.push(ChildType {
            descriptor,
            plan: plan.clone(),
        });
    }
    Ok(groups)
}

#[derive(Clone, Debug)]
pub(super) struct ListRequest {
    pub type_name: String,
    pub resource_model: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct GetRequest {
    pub type_name: String,
    pub identifier: String,
}

#[derive(Clone, Debug)]
struct ParentContext {
    resource_id: String,
    resource_kind: String,
    model: BTreeMap<String, String>,
    encoded_model: String,
    scope_parent: String,
}

fn parent_context(
    child: &ChildType<'_>,
    resource: &Resource,
) -> Result<ParentContext, AwsScanError> {
    let properties = resource
        .properties
        .get("provider_properties")
        .ok_or_else(|| invalid("parent read lacks sanitized provider properties"))?;
    let mut model = BTreeMap::new();
    for binding in &child.plan.bindings {
        let value = properties
            .pointer(&binding.source_pointer)
            .and_then(Value::as_str)
            .filter(|value| {
                !value.is_empty()
                    && value.len() <= MAX_IDENTIFIER_BYTES
                    && !value.chars().any(char::is_control)
            })
            .ok_or_else(|| invalid("parent read lacks a valid required context property"))?;
        model.insert(binding.model_property.clone(), value.to_owned());
    }
    let encoded_model =
        serde_json::to_string(&model).map_err(|error| invalid(error.to_string()))?;
    if encoded_model.len() > MAX_MODEL_BYTES {
        return Err(invalid("parent resource model exceeds its byte budget"));
    }
    let scope_parent = format!(
        "{}:{:x}",
        child.descriptor.type_name,
        Sha256::digest(encoded_model.as_bytes())
    );
    Ok(ParentContext {
        resource_id: resource.id.clone(),
        resource_kind: resource.kind.clone(),
        model,
        encoded_model,
        scope_parent,
    })
}

#[derive(Default)]
struct GroupLedger {
    models: BTreeSet<[u8; 32]>,
    items: BTreeSet<[u8; 32]>,
}

fn admit_model(ledger: &Mutex<GroupLedger>, scope_parent: &str) -> Result<(), AwsScanError> {
    let digest: [u8; 32] = Sha256::digest(scope_parent.as_bytes()).into();
    let mut state = ledger
        .lock()
        .map_err(|_| invalid("parent group ledger was poisoned"))?;
    if state.models.contains(&digest) {
        return Err(invalid(
            "different parent resources produced the same child list model",
        ));
    }
    if state.models.len() >= MAX_MODELS_PER_ROOT {
        return Err(AwsScanError::ChildLimit);
    }
    state.models.insert(digest);
    Ok(())
}

fn admit_children(
    ledger: &Mutex<GroupLedger>,
    descriptor: &ResourceType,
    output: &ListResourcesOutput,
) -> Result<(), AwsScanError> {
    let mut page = BTreeSet::new();
    for item in output.resource_descriptions() {
        let mut digest = Sha256::new();
        digest.update(descriptor.type_name.as_bytes());
        digest.update([0]);
        digest.update(identifier(item)?.as_bytes());
        page.insert(<[u8; 32]>::from(digest.finalize()));
    }
    let mut state = ledger
        .lock()
        .map_err(|_| invalid("parent group ledger was poisoned"))?;
    if state.items.len().saturating_add(page.len()) > MAX_CHILDREN {
        return Err(AwsScanError::ChildLimit);
    }
    if page.iter().any(|item| state.items.contains(item)) {
        return Err(invalid("child identifier repeated across parent scopes"));
    }
    state.items.extend(page);
    Ok(())
}

#[derive(Default)]
struct DiscoveryState {
    exhausted: AtomicBool,
    missing: AtomicBool,
    ledger: Mutex<GroupLedger>,
}

pub(super) async fn collect_group_with<L, LFut, G, GFut>(
    ctx: &Context,
    group: &RootGroup<'_>,
    list: L,
    get: G,
) -> Result<(), AwsScanError>
where
    L: Fn(ListRequest) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    if group.children.is_empty() {
        return collect_type_with(
            ctx,
            group.descriptor,
            |token| {
                list(ListRequest {
                    type_name: group.descriptor.type_name.clone(),
                    resource_model: None,
                    token,
                })
            },
            |identifier| {
                get(GetRequest {
                    type_name: group.descriptor.type_name.clone(),
                    identifier,
                })
            },
        )
        .await;
    }
    let state = DiscoveryState::default();
    collect_parent_pages(ctx, group, &state, &list, &get).await?;
    finish_discovery(ctx, group, &state).await
}

async fn collect_parent_pages<L, LFut, G, GFut>(
    ctx: &Context,
    group: &RootGroup<'_>,
    state: &DiscoveryState,
    list: &L,
    get: &G,
) -> Result<(), AwsScanError>
where
    L: Fn(ListRequest) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    let identities = Mutex::new(IdentityLedger::default());
    paginate(
        ctx,
        ctx.scope(
            "cloud_control_list",
            Some(&group.descriptor.type_name),
            Source::CloudControl,
        ),
        |token, budget| {
            let identities = &identities;
            async move {
                check_cancelled(ctx)?;
                let output = list(ListRequest {
                    type_name: group.descriptor.type_name.clone(),
                    resource_model: None,
                    token,
                })
                .await?;
                remember_list_page(group.descriptor, &output, identities, budget)?;
                collect_bounded(
                    output.resource_descriptions(),
                    READ_CONCURRENCY,
                    |description| collect_parent_item(ctx, group, state, description, list, get),
                )
                .await?;
                if output.next_token().is_none() {
                    state.exhausted.store(true, Ordering::Relaxed);
                }
                Ok(PageBody {
                    listed_count: output.resource_descriptions().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

fn check_cancelled(ctx: &Context) -> Result<(), AwsScanError> {
    if ctx.executor.is_cancelled() {
        Err(ApiError::Cancelled.into())
    } else {
        Ok(())
    }
}

async fn collect_parent_item<L, LFut, G, GFut>(
    ctx: &Context,
    group: &RootGroup<'_>,
    state: &DiscoveryState,
    listed: &ResourceDescription,
    list: &L,
    get: &G,
) -> Result<(), AwsScanError>
where
    L: Fn(ListRequest) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    check_cancelled(ctx)?;
    let id = identifier(listed)?;
    let parent_scope = format!(
        "{}:{:x}",
        group.descriptor.type_name,
        Sha256::digest(id.as_bytes())
    );
    let scope = ctx.scope(
        "cloud_control_get",
        Some(&parent_scope),
        Source::CloudControl,
    );
    let mapped = match get(GetRequest {
        type_name: group.descriptor.type_name.clone(),
        identifier: id.to_owned(),
    })
    .await
    {
        Ok(output) => map_get(ctx, group.descriptor, id, &output),
        Err(error) => Err(error.into()),
    };
    match mapped {
        Ok(body) => {
            let resource = body
                .resources
                .first()
                .ok_or_else(|| invalid("mapped parent has no resource"))?;
            let contexts: Vec<_> = group
                .children
                .iter()
                .map(|child| (child, parent_context(child, resource)))
                .collect();
            emit_child(ctx, &scope, body).await?;
            collect_bounded(
                contexts,
                CHILD_TYPE_CONCURRENCY,
                |(child, context)| async move {
                    match context {
                        Ok(context) => collect_child(ctx, child, &context, state, list, get).await,
                        Err(error) => {
                            state.missing.store(true, Ordering::Relaxed);
                            let failure_scope = ctx.scope(
                                "cloud_control_parent_context",
                                Some(&format!(
                                    "{}:{:x}",
                                    child.descriptor.type_name,
                                    Sha256::digest(id.as_bytes())
                                )),
                                Source::CloudControl,
                            );
                            ctx.failure(
                                &failure_scope,
                                "parent_context_unavailable",
                                error.to_string(),
                            )
                            .await
                        }
                    }
                },
            )
            .await?;
            Ok(())
        }
        Err(error) => {
            state.missing.store(true, Ordering::Relaxed);
            emit_failed_read(ctx, group.descriptor, listed, &scope, error, None).await
        }
    }
}

async fn finish_discovery(
    ctx: &Context,
    group: &RootGroup<'_>,
    state: &DiscoveryState,
) -> Result<(), AwsScanError> {
    check_cancelled(ctx)?;
    let complete =
        state.exhausted.load(Ordering::Relaxed) && !state.missing.load(Ordering::Relaxed);
    for child in &group.children {
        let scope = ctx.scope(
            "cloud_control_parent_discovery",
            Some(&child.descriptor.type_name),
            Source::CloudControl,
        );
        if complete {
            emit_child(ctx, &scope, PageBody::default()).await?;
        } else {
            ctx.failure(&scope, "parent_discovery_incomplete",
                "parent enumeration or a required parent read did not complete; child inventory may be incomplete").await?;
        }
    }
    Ok(())
}

async fn collect_child<L, LFut, G, GFut>(
    ctx: &Context,
    child: &ChildType<'_>,
    parent: &ParentContext,
    state: &DiscoveryState,
    list: &L,
    get: &G,
) -> Result<(), AwsScanError>
where
    L: Fn(ListRequest) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    check_cancelled(ctx)?;
    let scope = ctx.scope(
        "cloud_control_list",
        Some(&parent.scope_parent),
        Source::CloudControl,
    );
    if let Err(error) = admit_model(&state.ledger, &parent.scope_parent) {
        state.missing.store(true, Ordering::Relaxed);
        return ctx
            .failure(&scope, error_category(&error), error.to_string())
            .await;
    }
    let identities = Mutex::new(IdentityLedger::default());
    paginate(ctx, scope, |token, budget| {
        let identities = &identities;
        async move {
            check_cancelled(ctx)?;
            let output = list(ListRequest {
                type_name: child.descriptor.type_name.clone(),
                resource_model: Some(parent.encoded_model.clone()),
                token,
            })
            .await?;
            remember_list_page(child.descriptor, &output, identities, budget)?;
            admit_children(&state.ledger, child.descriptor, &output)?;
            collect_bounded(
                output.resource_descriptions(),
                READ_CONCURRENCY,
                |description| collect_child_item(ctx, child, parent, description, get),
            )
            .await?;
            Ok(PageBody {
                listed_count: output.resource_descriptions().len(),
                next_token: output.next_token().map(str::to_owned),
                ..PageBody::default()
            })
        }
    })
    .await?;
    check_cancelled(ctx)
}

async fn collect_child_item<G, GFut>(
    ctx: &Context,
    child: &ChildType<'_>,
    parent: &ParentContext,
    listed: &ResourceDescription,
    get: &G,
) -> Result<(), AwsScanError>
where
    G: Fn(GetRequest) -> GFut + Send + Sync,
    GFut: Future<Output = Result<GetResourceOutput, ApiError>> + Send,
{
    check_cancelled(ctx)?;
    let id = identifier(listed)?;
    let scope_parent = format!(
        "{}:{:x}",
        parent.scope_parent,
        Sha256::digest(id.as_bytes())
    );
    let scope = ctx.scope(
        "cloud_control_get",
        Some(&scope_parent),
        Source::CloudControl,
    );
    let mapped = match get(GetRequest {
        type_name: child.descriptor.type_name.clone(),
        identifier: id.to_owned(),
    })
    .await
    {
        Ok(output) => map_get(ctx, child.descriptor, id, &output)
            .and_then(|body| attach_parent(body, child, parent, true)),
        Err(error) => Err(error.into()),
    };
    match mapped {
        Ok(body) => emit_child(ctx, &scope, body).await,
        Err(error) => {
            emit_failed_read(
                ctx,
                child.descriptor,
                listed,
                &scope,
                error,
                Some((child, parent)),
            )
            .await
        }
    }
}

fn attach_parent(
    mut body: PageBody,
    child: &ChildType<'_>,
    parent: &ParentContext,
    complete: bool,
) -> Result<PageBody, AwsScanError> {
    let resource = body
        .resources
        .first_mut()
        .ok_or_else(|| invalid("child mapping has no resource"))?;
    let provider = resource
        .properties
        .get("provider_properties")
        .ok_or_else(|| invalid("child mapping lacks provider properties"))?;
    for (key, expected) in &parent.model {
        match provider.get(key) {
            Some(value) if value.as_str() == Some(expected) => {}
            None if !complete => {}
            _ => {
                return Err(invalid(
                    "child read does not match its requested parent resource model",
                ));
            }
        }
    }
    let properties = resource
        .properties
        .as_object_mut()
        .ok_or_else(|| invalid("child properties are not an object"))?;
    properties.insert("list_resource_model".into(), json!(parent.model));
    properties.insert("parent_resource_id".into(), json!(parent.resource_id));
    properties.insert("parent_type".into(), json!(child.plan.parent_type));
    if complete {
        if body.edges.len() >= MAX_RESOURCE_EDGES {
            return Err(invalid("child relationship cap exceeded"));
        }
        body.edges.push(Edge { from_id: resource.id.clone(), to_id: parent.resource_id.clone(),
            relation: child.plan.relation.clone(), source: resource.source, observed_at: resource.observed_at,
            properties: json!({"target_kind":parent.resource_kind,"cloudformation_type":child.descriptor.type_name,
                "parent_type":child.plan.parent_type,"resource_model":parent.model,
                "evidence":child.plan.evidence,"owner_resolution":"observed_parent_resource_identity"}),
        });
    }
    resource.validate()?;
    Ok(body)
}

async fn emit_failed_read(
    ctx: &Context,
    descriptor: &ResourceType,
    listed: &ResourceDescription,
    scope: &CollectionScope,
    error: AwsScanError,
    parent: Option<(&ChildType<'_>, &ParentContext)>,
) -> Result<(), AwsScanError> {
    if matches!(
        error,
        AwsScanError::ChannelClosed | AwsScanError::Api(ApiError::Cancelled)
    ) {
        return Err(error);
    }
    ctx.failure(scope, error_category(&error), error.to_string())
        .await?;
    let fallback =
        map_description(ctx, descriptor, identifier(listed)?, listed, false).and_then(|body| {
            match parent {
                Some((child, context)) => attach_parent(body, child, context, false),
                None => Ok(body),
            }
        });
    match fallback {
        Ok(body) => emit_child(ctx, scope, body).await,
        Err(error) => {
            ctx.failure(
                scope,
                "invalid_response",
                format!("list-only fallback could not be retained: {error}"),
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod wire_tests;
