use super::super::{RuntimeCatalogCompileError, model_path};
use crate::overlay::{Identity, IdentityChoice, IdentityField, IdentityLeaf, IdentityPredicate};
use inventory_core::catalog::{Operation, Service, Target};

pub(in crate::runtime_catalog) fn validate(service: Service<'_>, operation: Operation<'_>, item: Target, identity: &Identity) -> Result<(), RuntimeCatalogCompileError> {
    match identity {
        Identity::MapArn { path, resource_kind, request_url, .. } => {
            text_map(service, item, path.as_str())?;
            if matches!(resource_kind, crate::overlay::ArnResourceKind::SqsQueue) != request_url.is_some() {
                return Err(RuntimeCatalogCompileError::Capability);
            }
            request_url.as_ref().map_or(Ok(()), |path| text(service, target(service, operation.input()?, path.as_str())?))
        }
        Identity::CorrelatedKey(fields) => validate_fields(service, operation, item, fields),
        Identity::Choice(choice) => {
            choice.validate().map_err(|_| RuntimeCatalogCompileError::Capability)?;
            match choice {
                IdentityChoice::PreferArn { path, fallback, .. } => { text(service, target(service, item, path.as_str())?)?; leaf(service, operation, item, fallback) }
                IdentityChoice::Discriminated(branches) => branches.iter().try_for_each(|branch| {
                    match &branch.predicate {
                        IdentityPredicate::Present(path) | IdentityPredicate::TextEquals { path, .. } => text(service, target(service, item, path.as_str())?)?,
                        IdentityPredicate::RequestTextEquals { path, .. } => text(service, target(service, operation.input()?, path.as_str())?)?,
                    }
                    leaf(service, operation, item, &branch.identity)
                }),
            }
        }
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}
fn leaf(service: Service<'_>, operation: Operation<'_>, item: Target, identity: &IdentityLeaf) -> Result<(), RuntimeCatalogCompileError> {
    match identity {
        IdentityLeaf::Arn(path) => text(service, target(service, item, path.as_str())?),
        IdentityLeaf::Template(template) => super::super::plans::identity::fields(template).try_for_each(|path| scalar(service, target(service, item, path)?)),
        IdentityLeaf::Key(paths) => paths.iter().try_for_each(|path| scalar(service, target(service, item, path.as_str())?)),
        IdentityLeaf::Fields(fields) => validate_fields(service, operation, item, fields),
    }
}
fn validate_fields(service: Service<'_>, operation: Operation<'_>, item: Target, fields: &[IdentityField]) -> Result<(), RuntimeCatalogCompileError> {
    if fields.is_empty() || fields.len() > 64 { return Err(RuntimeCatalogCompileError::Capability) }
    fields.iter().try_for_each(|field| match field {
        IdentityField::Selected(path) => scalar(service, target(service, item, path.as_str())?),
        IdentityField::SelectedDigest(path) => digest_target(service, target(service, item, path.as_str())?),
        IdentityField::Request(path) => scalar(service, target(service, operation.input()?, path.as_str())?),
        IdentityField::SelectedArn(path) => text(service, target(service, item, path.as_str())?),
        IdentityField::RequestArn(path) => text(service, target(service, operation.input()?, path.as_str())?),
        IdentityField::SelectedInteger { path, minimum, maximum } =>
            integral(service, target(service, item, path.as_str())?, *minimum, *maximum),
        IdentityField::RequestInteger { path, minimum, maximum } =>
            integral(service, target(service, operation.input()?, path.as_str())?, *minimum, *maximum),
        IdentityField::SelectedStringList { path, max_items, max_item_bytes } => {
            if !(1..=64).contains(max_items) || !(1..=4096).contains(max_item_bytes) {
                return Err(RuntimeCatalogCompileError::Capability)
            }
            let Target::Shape(index) = target(service, item, path.as_str())? else {
                return Err(RuntimeCatalogCompileError::Capability)
            };
            let shape = service.shape(index)?.ok_or(RuntimeCatalogCompileError::Capability)?;
            if shape.kind()? != "list" { return Err(RuntimeCatalogCompileError::Capability) }
            text(service, shape.element()?.ok_or(RuntimeCatalogCompileError::Capability)?)
        }
        IdentityField::MapValue { path, .. } => text_map(service, item, path.as_str()),
        // Cross-plan context paths are checked against reviewed producer selectors by the runtime catalog dependency validator.
        IdentityField::Context { .. } => Ok(()),
    })
}
fn target(service: Service<'_>, root: Target, path: &str) -> Result<Target, RuntimeCatalogCompileError> {
    if path == "$" { Ok(root) } else { model_path::resolve(service, root, path).map(|(_, target)| target) }
}
fn text_map(service: Service<'_>, root: Target, path: &str) -> Result<(), RuntimeCatalogCompileError> {
    let Target::Shape(index) = target(service, root, path)? else { return Err(RuntimeCatalogCompileError::Capability) };
    let shape = service.shape(index)?.ok_or(RuntimeCatalogCompileError::Capability)?;
    if shape.kind()? != "map" { return Err(RuntimeCatalogCompileError::Capability) }
    text(service, shape.map_key()?.ok_or(RuntimeCatalogCompileError::Capability)?)?;
    text(service, shape.map_value()?.ok_or(RuntimeCatalogCompileError::Capability)?)
}
fn text(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    if matches!(model_path::kind(service, target)?, "string" | "enum") { Ok(()) } else { Err(RuntimeCatalogCompileError::Capability) }
}
fn scalar(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    if matches!(model_path::kind(service, target)?, "string" | "enum" | "boolean" | "byte" | "short" | "integer" | "long") { Ok(()) } else { Err(RuntimeCatalogCompileError::Capability) }
}

fn integral(service: Service<'_>, target: Target, minimum: i64, maximum: i64) -> Result<(), RuntimeCatalogCompileError> {
    let (low, high) = match model_path::kind(service, target)? {
        "byte" => (i64::from(i8::MIN), i64::from(i8::MAX)),
        "short" => (i64::from(i16::MIN), i64::from(i16::MAX)),
        "integer" => (i64::from(i32::MIN), i64::from(i32::MAX)),
        "long" => (i64::MIN, i64::MAX),
        _ => return Err(RuntimeCatalogCompileError::Capability),
    };
    if minimum <= maximum && minimum >= low && maximum <= high { Ok(()) }
    else { Err(RuntimeCatalogCompileError::Capability) }
}

fn digest_target(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    if matches!(model_path::kind(service, target)?, "string" | "enum" | "intEnum" | "boolean"
        | "byte" | "short" | "integer" | "long" | "float" | "double" | "bigInteger"
        | "bigDecimal" | "blob" | "timestamp" | "structure" | "union" | "list" | "map") {
        Ok(())
    } else { Err(RuntimeCatalogCompileError::Capability) }
}
