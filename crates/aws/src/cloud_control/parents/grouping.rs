use super::*;

pub(in super::super) fn groups(types: &[ResourceType]) -> Result<Vec<RootGroup<'_>>, AwsScanError> {
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

pub(in super::super) fn parent_context(
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
        base16ct::HexDisplay(&Sha256::digest(encoded_model.as_bytes()))
    );
    Ok(ParentContext {
        resource_id: resource.id.clone(),
        resource_kind: resource.kind.clone(),
        model,
        encoded_model,
        scope_parent,
    })
}

pub(in super::super) fn admit_model(
    ledger: &Mutex<GroupLedger>,
    scope_parent: &str,
) -> Result<(), AwsScanError> {
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

pub(in super::super) fn admit_children(
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
