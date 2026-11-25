use super::*;
use crate::runtime_catalog::ServiceIndex;
use crate::runtime_catalog::selection as runtime_selection;

pub(super) fn encode_plan(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    let services = context.services;
    let head = head(prepared, overlay, callable.operation())?;
    let body = overlay_fields(prepared, overlay, services)?;
    let tail = tail(review, callable, context, prepared)?;
    let extensions = extensions(review, callable, context, prepared)?;
    encode::map(head.into_iter().chain(body).chain(tail).chain(extensions))
}

fn extensions(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<impl Iterator<Item = (u32, Value)>, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    let tags = tags::compile(prepared.service, prepared.operation, overlay)?;
    let routing = routing(callable, context, prepared)?;
    let capabilities = capabilities::compile(prepared.service, prepared.operation, overlay)?;
    let discovery = discovery(callable, context)?;
    let map_exclusions = map_exclusions(prepared, overlay)?;
    let row_policy = row_policy(callable, context, prepared)?;
    let emissions = emissions(callable, context, prepared)?;
    let emit = emit_value(callable, review, prepared)?;
    Ok(tags
        .map(|value| (14, value))
        .into_iter()
        .chain(routing)
        .chain(capabilities)
        .chain(discovery)
        .chain(map_exclusions)
        .chain(member_exclusions::compile(overlay, context.services)?)
        .chain(row_policy)
        .chain(emissions)
        .chain(Some((25, emit))))
}

fn emit_value(
    callable: &CallableOperation,
    review: &crate::overlay::ReviewedService,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let has_bound_expansion = review.callable().iter().any(|candidate| {
        candidate.class() == crate::overlay::EffectiveClass::Expand
            && has_binding_to(candidate, callable)
    });
    emission_value(
        callable.plan().emission,
        callable.plan().emission_explicit,
        callable.class() == crate::overlay::EffectiveClass::Enumerate,
        has_bound_expansion,
        || should_emit_work(callable, review, prepared),
    )
}

fn emission_value(
    emission: crate::overlay::Emission,
    is_explicit: bool,
    is_enumerate: bool,
    has_bound_expansion: bool,
    infer: impl FnOnce() -> Result<bool, RuntimeCatalogCompileError>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if emission == crate::overlay::Emission::WorkOnly {
        return if is_enumerate && has_bound_expansion {
            Ok(Value::Int(1))
        } else {
            Err(RuntimeCatalogCompileError::Path {
                path: "emit work-only requires an enumerate operation with a bound expansion"
                    .to_owned(),
            })
        };
    }
    if emission == crate::overlay::Emission::EmitMetadata && !is_explicit && is_enumerate {
        return Ok(Value::Int(if infer()? { 1 } else { 0 }));
    }
    Ok(Value::Int(0))
}

fn should_emit_work(
    callable: &CallableOperation,
    review: &crate::overlay::ReviewedService,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<bool, RuntimeCatalogCompileError> {
    let Some(selector) = callable.plan().items.as_ref() else {
        return Ok(false);
    };
    let output = prepared.operation.output()?;
    let item_target = resolve_item_target(prepared.service, output, Some(selector))?;
    if !item_is_scalar_or_identifier_only(
        prepared.service,
        item_target,
        callable.plan().identity.as_ref(),
    )? {
        return Ok(false);
    }
    let mut expansions = review.callable().iter().filter(|candidate| {
        candidate.class() == crate::overlay::EffectiveClass::Expand
            && has_binding_to(candidate, callable)
    });
    let Some(expansion) = expansions.next() else {
        return Ok(false);
    };
    if expansions.next().is_some() {
        return Ok(false);
    }
    let binding = expansion_binding(prepared.service, expansion)?;
    Ok(crate::overlay::infer_work_only(true, &[binding]))
}

fn expansion_binding(
    service: inventory_core::catalog::Service<'_>,
    expansion: &CallableOperation,
) -> Result<crate::overlay::ExpansionBinding, RuntimeCatalogCompileError> {
    let selector = expansion.plan().items.as_ref();
    let Some(selector) = selector else {
        return Ok(unproven_expansion_binding());
    };
    let operation = find_operation(service, expansion.operation())?;
    let target = resolve_item_target(service, operation.output()?, Some(selector))?;
    let output_kind = match selector {
        crate::overlay::ItemSelector::Explicit { kind, .. } => *kind,
        crate::overlay::ItemSelector::Root | crate::overlay::ItemSelector::Member(_) => {
            modeled_selection_kind(service, target)?
        }
    };
    Ok(crate::overlay::ExpansionBinding {
        output_kind: Some(output_kind),
    })
}

fn unproven_expansion_binding() -> crate::overlay::ExpansionBinding {
    crate::overlay::ExpansionBinding { output_kind: None }
}

fn find_operation<'a>(
    service: inventory_core::catalog::Service<'a>,
    name: &str,
) -> Result<inventory_core::catalog::Operation<'a>, RuntimeCatalogCompileError> {
    service
        .operations()?
        .try_fold(
            None,
            |found, operation| -> Result<_, RuntimeCatalogCompileError> {
                let operation = operation?;
                if found.is_some() || operation.name()? != name {
                    Ok(found)
                } else {
                    Ok(Some(operation))
                }
            },
        )?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: name.to_owned(),
        })
}

fn resolve_item_target(
    service: inventory_core::catalog::Service<'_>,
    output: inventory_core::catalog::Target,
    selector: Option<&crate::overlay::ItemSelector>,
) -> Result<inventory_core::catalog::Target, RuntimeCatalogCompileError> {
    runtime_selection::target(service, output, selector)
}

fn modeled_selection_kind(
    service: inventory_core::catalog::Service<'_>,
    target: inventory_core::catalog::Target,
) -> Result<crate::overlay::SelectionKind, RuntimeCatalogCompileError> {
    let kind = match target {
        inventory_core::catalog::Target::Prelude(kind) => kind,
        inventory_core::catalog::Target::Shape(index) => service
            .shape(index)?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: "item target".to_owned(),
            })?
            .kind()?,
        _ => {
            return Err(RuntimeCatalogCompileError::Path {
                path: "item target".to_owned(),
            });
        }
    };
    Ok(match kind {
        "structure" => crate::overlay::SelectionKind::Structure,
        "list" => crate::overlay::SelectionKind::List,
        "map" => crate::overlay::SelectionKind::MapEntries,
        _ => crate::overlay::SelectionKind::Scalar,
    })
}

fn item_is_scalar_or_identifier_only(
    service: inventory_core::catalog::Service<'_>,
    target: inventory_core::catalog::Target,
    identity: Option<&crate::overlay::Identity>,
) -> Result<bool, RuntimeCatalogCompileError> {
    let inventory_core::catalog::Target::Shape(index) = target else {
        return Ok(matches!(
            target,
            inventory_core::catalog::Target::Prelude(kind) if is_scalar_kind(kind)
        ));
    };
    let shape = service
        .shape(index)?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: "item target".to_owned(),
        })?;
    let kind = shape.kind()?;
    if is_scalar_kind(kind) {
        return Ok(true);
    }
    if kind != "structure" {
        return Ok(false);
    }
    let Some(identity) = identity else {
        return Ok(false);
    };
    let (all_identifiers, has_members) =
        shape
            .members()?
            .try_fold((true, false), |(all_identifiers, _), member| {
                let member = member?;
                Ok::<_, inventory_core::catalog::CatalogError>((
                    all_identifiers && identity_uses_member(identity, member.name()?),
                    true,
                ))
            })?;
    Ok(all_identifiers && has_members)
}

fn is_scalar_kind(kind: &str) -> bool {
    matches!(
        kind,
        "string"
            | "blob"
            | "boolean"
            | "byte"
            | "short"
            | "integer"
            | "long"
            | "float"
            | "double"
            | "bigInteger"
            | "bigDecimal"
            | "timestamp"
            | "enum"
            | "intEnum"
    )
}

fn identity_uses_member(identity: &crate::overlay::Identity, member: &str) -> bool {
    use crate::overlay::{Identity, IdentityField};
    match identity {
        Identity::Arn(path) | Identity::MapArn { path, .. } => member_path_matches(path, member),
        Identity::Key(paths) => paths.iter().any(|path| member_path_matches(path, member)),
        Identity::Template(template) => super::super::identity::fields(template)
            .any(|path| member_path_matches_text(path, member)),
        Identity::CorrelatedKey(fields) => fields.iter().any(|field| match field {
            IdentityField::Selected(path)
            | IdentityField::SelectedDigest(path)
            | IdentityField::SelectedArn(path)
            | IdentityField::SelectedInteger { path, .. }
            | IdentityField::SelectedStringList { path, .. }
            | IdentityField::MapValue { path, .. } => member_path_matches(path, member),
            IdentityField::Context {
                source: crate::overlay::ContextSource::SelectedItem,
                path,
                ..
            } => member_path_matches(path, member),
            _ => false,
        }),
        Identity::Choice(choice) => match choice {
            crate::overlay::IdentityChoice::PreferArn { path, fallback, .. } => {
                member_path_matches(path, member) || leaf_uses_member(fallback, member)
            }
            crate::overlay::IdentityChoice::Discriminated(branches) => branches
                .iter()
                .any(|branch| leaf_uses_member(&branch.identity, member)),
        },
    }
}

fn member_path_matches(path: &crate::overlay::MemberPath, member: &str) -> bool {
    member_path_matches_text(path.as_str(), member)
}

fn member_path_matches_text(path: &str, member: &str) -> bool {
    path == member
        || path
            .strip_prefix(member)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

fn leaf_uses_member(leaf: &crate::overlay::IdentityLeaf, member: &str) -> bool {
    use crate::overlay::{IdentityField, IdentityLeaf};
    let field_matches = |field: &IdentityField| match field {
        IdentityField::Selected(path)
        | IdentityField::SelectedDigest(path)
        | IdentityField::SelectedArn(path)
        | IdentityField::SelectedInteger { path, .. }
        | IdentityField::SelectedStringList { path, .. }
        | IdentityField::MapValue { path, .. } => member_path_matches(path, member),
        IdentityField::Context {
            source: crate::overlay::ContextSource::SelectedItem,
            path,
            ..
        } => member_path_matches(path, member),
        _ => false,
    };
    match leaf {
        IdentityLeaf::Arn(path) => member_path_matches(path, member),
        IdentityLeaf::Key(paths) => paths.iter().any(|path| member_path_matches(path, member)),
        IdentityLeaf::Template(template) => super::super::identity::fields(template)
            .any(|path| member_path_matches_text(path, member)),
        IdentityLeaf::Fields(fields) => fields.iter().any(field_matches),
    }
}

fn has_binding_to(expansion: &CallableOperation, enumerate: &CallableOperation) -> bool {
    expansion.plan().bind.iter().any(|(_, binding)| {
        matches!(binding, crate::overlay::Binding::From { operation, .. }
            if operation.as_str() == enumerate.operation())
            || matches!(binding, crate::overlay::Binding::Context { operation, .. }
                if operation.as_str() == enumerate.operation())
            || matches!(binding, crate::overlay::Binding::FromBranch { operation, .. }
                if operation.as_str() == enumerate.operation())
    })
}

fn routing(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<impl IntoIterator<Item = (u32, Value)>, RuntimeCatalogCompileError> {
    routing::compile(
        prepared.service,
        prepared.operation,
        callable.plan(),
        context.checked,
        context.services,
    )
}

fn row_policy(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<impl IntoIterator<Item = (u32, Value)>, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    row_policy::compile(
        prepared.service,
        prepared.operation,
        &overlay.row_policy,
        overlay.items.as_ref(),
        context.checked,
        callable.service(),
        context.services,
    )
}

fn head(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    operation: &str,
) -> Result<[(u32, Value); 4], RuntimeCatalogCompileError> {
    Ok([
        (1, Value::Text(prepared.service_name.to_owned())),
        (2, Value::Text(operation.to_owned())),
        (3, Value::Text(prepared.kind.to_owned())),
        (4, Value::Int(scope(overlay.scope)?)),
    ])
}

fn tail(
    review: &crate::overlay::ReviewedService,
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<[(u32, Value); 4], RuntimeCatalogCompileError> {
    let purpose = purpose(callable, context, prepared)?;
    Ok([
        (10, endpoint_value(prepared.endpoint, callable.operation())?),
        (11, review_value(review, callable.origin())?),
        (12, purpose),
        (13, Value::Int(prepared.protocol)),
    ])
}

fn purpose(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let overlay = callable.plan();
    match tag_endpoint::compile(
        prepared.service,
        prepared.operation,
        overlay,
        context.services,
    )? {
        Some(purpose) => Ok(purpose),
        None if overlay.endpoint_discovery => encode::map([(1, Value::Int(4))]),
        None => purpose_value(
            callable.service(),
            callable.operation(),
            context.kinds,
            context.services,
        ),
    }
}

fn discovery(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
) -> Result<[(u32, Value); 2], RuntimeCatalogCompileError> {
    Ok([
        (19, selection::emission(callable.plan())),
        (
            20,
            selection::context_projection(
                callable.service(),
                callable.operation(),
                context.checked,
                context.services,
            )?,
        ),
    ])
}

fn map_exclusions(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if overlay.exclude_map_keys.is_empty() {
        return Ok(None);
    }
    let item = crate::runtime_catalog::selection::target(
        prepared.service,
        prepared.operation.output()?,
        overlay.items.as_ref(),
    )?;
    Ok(Some((
        21,
        super::super::map_exclusions::encode(prepared.service, item, overlay)?,
    )))
}

fn emissions(
    callable: &CallableOperation,
    context: &PlanContext<'_, '_>,
    prepared: &PreparedPlan<'_, '_>,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if callable.plan().additional_resources.is_empty() {
        return Ok(None);
    }
    let source = emissions::Callable {
        service_name: callable.service(),
        operation_name: callable.operation(),
        primary_kind: prepared.kind,
        service: prepared.service,
        operation: prepared.operation,
    };
    Ok(Some((
        23,
        emissions::compile(source, callable.plan(), context.kinds, context.services)?,
    )))
}

fn overlay_fields(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &ServiceIndex<'_>,
) -> Result<[(u32, Value); 5], RuntimeCatalogCompileError> {
    let identity = overlay_identity(prepared, overlay, services)?;
    Ok([
        (5, item_value(overlay.items.as_ref(), services)?),
        (6, identity),
        (
            7,
            binding_values(prepared.service_name, prepared.operation, overlay, services)?,
        ),
        (8, pagination(prepared, overlay, services)?),
        (9, exclusions(overlay, services)?),
    ])
}

fn overlay_identity(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if overlay.emission == crate::overlay::Emission::DiscoveryOnly
        || overlay.tag_endpoint.is_some()
        || overlay.endpoint_discovery
    {
        encode::map([(1, Value::Int(2)), (2, Value::List(Vec::new()))])
    } else {
        identity_value(overlay.identity.as_ref(), prepared.service_name, services)
    }
}

fn pagination(
    prepared: &PreparedPlan<'_, '_>,
    overlay: &OperationOverlay,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    compile_page(
        overlay,
        prepared.service,
        prepared.operation,
        prepared.protocol,
        services,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::{Emission, Identity, ItemSelector, MemberPath};
    use inventory_core::catalog::{Catalog, Service, Target};
    use std::error::Error;

    type TestResult<T = ()> = Result<T, Box<dyn Error>>;

    const MODEL: &str = r#"{"shapes":{
        "n#Svc":{"type":"service","version":"2020-01-01","traits":{"aws.protocols#restXml":{}}},
        "n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},
            "traits":{"smithy.api#readonly":{},"smithy.api#http":{"method":"GET","uri":"/things"}}},
        "n#Input":{"type":"structure","members":{}},
        "n#Output":{"type":"structure","members":{
            "ItemsBad":{"target":"n#Bad"},"ItemsId":{"target":"n#Identifier"},
            "ItemsList":{"target":"n#ListOfLists"},"ItemsNamed":{"target":"n#NamedString"},
            "ItemsPrelude":{"target":"smithy.api#String"}}},
        "n#NamedString":{"type":"string"},
        "n#ListOfLists":{"type":"list","member":{"target":"n#Rows"}},
        "n#Rows":{"type":"list","member":{"target":"smithy.api#String"}},
        "n#Bad":{"type":"structure","members":{"Name":{"target":"smithy.api#String"}}},
        "n#Identifier":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
    }}"#;

    fn with_fixture(
        run: impl FnOnce(Service<'_>, inventory_core::catalog::Operation<'_>) -> TestResult,
    ) -> TestResult {
        let directory = tempfile::tempdir()?;
        let service_dir = directory.path().join("models/example/service/2020-01-01");
        std::fs::create_dir_all(&service_dir)?;
        std::fs::write(service_dir.join("example.json"), MODEL)?;
        let registry = crate::smithy::Registry::parse(
            "Id\nItemsBad\nItemsId\nItemsList\nItemsNamed\nItemsPrelude\nName\nOutput\nRows\n",
        )?;
        let bytes = crate::smithy::build_catalog(directory.path(), &registry, "fixture")?;
        let catalog = Catalog::borrow(&bytes)?;
        let service = catalog
            .service("example")?
            .ok_or("missing fixture service")?;
        let operation = service.operations()?.next().ok_or("missing operation")??;
        run(service, operation)
    }

    fn item_target(
        service: Service<'_>,
        operation: inventory_core::catalog::Operation<'_>,
        path: &str,
    ) -> TestResult<Target> {
        let output = operation.output()?;
        let selector = ItemSelector::Member(MemberPath::new(path)?);
        Ok(resolve_item_target(service, output, Some(&selector))?)
    }
    fn scalar_item_target(
        service: Service<'_>,
        operation: inventory_core::catalog::Operation<'_>,
        path: &str,
    ) -> TestResult<Target> {
        let output = operation.output()?;
        let selector = ItemSelector::Explicit {
            path: MemberPath::new(path)?,
            kind: crate::overlay::SelectionKind::Scalar,
        };
        Ok(resolve_item_target(service, output, Some(&selector))?)
    }

    fn structure_expansion() -> [crate::overlay::ExpansionBinding; 1] {
        [crate::overlay::ExpansionBinding {
            output_kind: Some(crate::overlay::SelectionKind::Structure),
        }]
    }

    #[test]
    fn given_named_string_item_when_inferred_then_work_only() -> TestResult {
        with_fixture(|service, operation| {
            let target = scalar_item_target(service, operation, "ItemsNamed")?;
            let item_qualifies = item_is_scalar_or_identifier_only(service, target, None)?;
            assert!(crate::overlay::infer_work_only(
                item_qualifies,
                &structure_expansion()
            ));
            Ok(())
        })
    }

    #[test]
    fn given_prelude_scalar_item_when_inferred_then_work_only() -> TestResult {
        with_fixture(|service, operation| {
            let target = scalar_item_target(service, operation, "ItemsPrelude")?;
            let item_qualifies = item_is_scalar_or_identifier_only(service, target, None)?;
            assert!(crate::overlay::infer_work_only(
                item_qualifies,
                &structure_expansion()
            ));
            Ok(())
        })
    }

    #[test]
    fn given_list_item_when_inferred_then_resources_remain_enabled() -> TestResult {
        with_fixture(|service, operation| {
            let target = item_target(service, operation, "ItemsList")?;
            let item_qualifies = item_is_scalar_or_identifier_only(service, target, None)?;
            assert!(!crate::overlay::infer_work_only(
                item_qualifies,
                &structure_expansion()
            ));
            Ok(())
        })
    }

    #[test]
    fn given_one_member_not_in_identity_when_inferred_then_resources_remain_enabled() -> TestResult
    {
        with_fixture(|service, operation| {
            let target = item_target(service, operation, "ItemsBad")?;
            let identity = Identity::Key(vec![MemberPath::new("Id")?]);
            let item_qualifies =
                item_is_scalar_or_identifier_only(service, target, Some(&identity))?;
            assert!(!crate::overlay::infer_work_only(
                item_qualifies,
                &structure_expansion()
            ));
            Ok(())
        })
    }

    #[test]
    fn given_identifier_only_structure_when_inferred_then_work_only() -> TestResult {
        with_fixture(|service, operation| {
            let target = item_target(service, operation, "ItemsId")?;
            let identity = Identity::Key(vec![MemberPath::new("Id")?]);
            let item_qualifies =
                item_is_scalar_or_identifier_only(service, target, Some(&identity))?;
            assert!(crate::overlay::infer_work_only(
                item_qualifies,
                &structure_expansion()
            ));
            Ok(())
        })
    }

    #[test]
    fn given_missing_item_member_when_resolved_then_compile_error_propagates() -> TestResult {
        with_fixture(|service, operation| {
            let error = item_target(service, operation, "Missing")
                .err()
                .ok_or("resolved missing path")?;
            assert!(error.downcast_ref::<RuntimeCatalogCompileError>().is_some());
            Ok(())
        })
    }

    #[test]
    fn given_explicit_resources_and_qualifying_inference_when_compiled_then_resources_win() {
        let value = emission_value(Emission::EmitMetadata, true, true, true, || Ok(true));
        assert!(matches!(value, Ok(Value::Int(0))));
    }
    #[test]
    fn given_valid_explicit_work_only_when_compiled_then_work_is_selected() {
        let value = emission_value(Emission::WorkOnly, true, true, true, || Ok(false));
        assert!(matches!(value, Ok(Value::Int(1))));
    }

    #[test]
    fn given_work_only_for_non_enumerate_when_compiled_then_compile_fails() {
        let value = emission_value(Emission::WorkOnly, true, false, true, || Ok(false));
        assert!(matches!(
            value,
            Err(RuntimeCatalogCompileError::Path { .. })
        ));
    }

    #[test]
    fn given_work_only_without_bound_expansion_when_compiled_then_compile_fails() {
        let value = emission_value(Emission::WorkOnly, true, true, false, || Ok(false));
        assert!(matches!(
            value,
            Err(RuntimeCatalogCompileError::Path { .. })
        ));
    }

    #[test]
    fn given_expansion_without_item_selector_when_inferred_then_work_is_not_assumed() {
        let expansion = unproven_expansion_binding();
        assert!(!crate::overlay::infer_work_only(true, &[expansion]));
    }

    #[test]
    fn given_model_resolution_error_when_inferred_then_compile_error_propagates() {
        let result = emission_value(Emission::EmitMetadata, false, true, true, || {
            Err(RuntimeCatalogCompileError::Path {
                path: "modeled item".to_owned(),
            })
        });
        assert!(
            matches!(result, Err(RuntimeCatalogCompileError::Path { path }) if path == "modeled item")
        );
    }
}
