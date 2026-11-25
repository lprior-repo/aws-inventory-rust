mod emission;
mod validation;
mod validation_context;
pub(super) use emission::active_kind;
pub(super) use validation::validate_reviewed_paths;
pub(super) use validation_context::ValidationContext;
use inventory_core::{
    catalog::{Service, Shape, Target},
    record::FieldId,
};
use super::RuntimeCatalogCompileError;

pub(super) fn resolve(
    service: Service<'_>,
    mut target: Target,
    path: &str,
) -> Result<(Vec<FieldId>, Target), RuntimeCatalogCompileError> {
    let segments = path.split('.').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > 8 || segments.iter().any(|part| part.is_empty()) {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        });
    }
    let mut fields = Vec::with_capacity(segments.len());
    let result = segments.iter().try_for_each(|segment| {
        let shape = shape_for(service, target)?;
        let member = member(shape, segment)?;
        fields.push(member.id);
        target = member.target;
        Ok::<_, RuntimeCatalogCompileError>(())
    });
    match result {
        Ok(()) => Ok((fields, target)),
        Err(RuntimeCatalogCompileError::Path { path: _ }) => {
            Err(RuntimeCatalogCompileError::Path { path: path.to_owned() })
        }
        Err(error) => Err(error),
    }
}

pub(super) fn terminal_member<'a>(
    service: Service<'a>,
    target: Target,
    path: &str,
) -> Result<(Vec<FieldId>, inventory_core::catalog::Member<'a>), RuntimeCatalogCompileError> {
    let segments = path.split('.').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > 8 {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        });
    }
    let result = terminal_member_segments(service, target, &segments);
    match result {
        Ok(value) => Ok(value),
        Err(RuntimeCatalogCompileError::Path { path: _ }) => {
            Err(RuntimeCatalogCompileError::Path { path: path.to_owned() })
        }
        Err(error) => Err(error),
    }
}

fn terminal_member_segments<'a>(
    service: Service<'a>,
    target: Target,
    segments: &[&str],
) -> Result<(Vec<FieldId>, inventory_core::catalog::Member<'a>), RuntimeCatalogCompileError> {
    let (head, tail) = segments
        .split_first()
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: "terminal_member_segments".to_owned(),
        })?;
    let current = member(shape_for(service, target)?, head)?;
    if tail.is_empty() {
        return Ok((vec![current.id], current));
    }
    let (path, terminal) = terminal_member_segments(service, current.target, tail)?;
    Ok((std::iter::once(current.id).chain(path).collect(), terminal))
}

pub(super) fn optional_resolve(
    service: Service<'_>,
    target: Target,
    path: &str,
) -> Result<Option<Target>, RuntimeCatalogCompileError> {
    path.split('.').try_fold(Some(target), |target, segment| {
        let Some(target) = target else {
            return Ok(None);
        };
        match kind(service, target)? {
            "string" | "enum" | "intEnum" | "boolean" | "byte" | "short" | "integer" | "long"
            | "float" | "double" | "bigInteger" | "bigDecimal" | "timestamp" | "blob" => {
                return Ok(None);
            }
            "structure" | "union" => {}
            kind => {
                return Err(RuntimeCatalogCompileError::Path {
                    path: format!("optional_resolve kind {kind}"),
                })
            }
        }
        let shape = shape_for(service, target)?;
        shape
            .members()?
            .try_fold(None, |found, candidate| {
                let candidate = candidate?;
                if candidate.name()? == segment {
                    Ok(Some(candidate.target))
                } else {
                    Ok(found)
                }
            })
            .map_err(RuntimeCatalogCompileError::Catalog)
    })
}

pub(super) fn list_element(
    service: Service<'_>,
    target: Target,
) -> Result<Target, RuntimeCatalogCompileError> {
    let list = shape_for(service, target)?;
    if list.kind()? != "list" {
        return Err(RuntimeCatalogCompileError::Path {
            path: "list_element kind".to_owned(),
        });
    }
    list.element()
        ?.ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: "list_element element".to_owned(),
        })
}

pub(super) fn kind(
    service: Service<'_>,
    target: Target,
) -> Result<&'static str, RuntimeCatalogCompileError> {
    match target {
        Target::Prelude(kind) => Ok(kind),
        Target::Shape(_) => Ok(shape_for(service, target)?.kind()?),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "kind target".to_owned(),
        }),
    }
}

pub(super) fn string_target(
    service: Service<'_>,
    target: Target,
) -> Result<(), RuntimeCatalogCompileError> {
    let kind = match target {
        Target::Prelude(kind) => kind,
        Target::Shape(_) => kind(service, target)?,
        _ => {
            return Err(RuntimeCatalogCompileError::Path {
                path: "string_target target".to_owned(),
            })
        }
    };
    if matches!(kind, "string" | "enum") {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Path {
            path: format!("string_target kind {kind}"),
        })
    }
}

fn shape_for<'a>(
    service: Service<'a>,
    target: Target,
) -> Result<Shape<'a>, RuntimeCatalogCompileError> {
    match target {
        Target::Shape(index) => service
            .shape(index)?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: "shape_for index".to_owned(),
            }),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "shape_for target".to_owned(),
        }),
    }
}

fn member<'a>(
    shape: Shape<'a>,
    name: &str,
) -> Result<inventory_core::catalog::Member<'a>, RuntimeCatalogCompileError> {
    shape
        .members()?
        .try_fold(None, |found, candidate| {
            let candidate = candidate?;
            if candidate.name()? != name {
                return Ok(found);
            }
            if found.is_some() {
                return Err(inventory_core::catalog::CatalogError::Reference);
            }
            Ok(Some(candidate))
        })?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: "member name".to_owned(),
        })
}