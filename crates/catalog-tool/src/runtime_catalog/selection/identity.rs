use super::super::{RuntimeCatalogCompileError, model_path};
use crate::overlay::{Identity, IdentityChoice, IdentityField, IdentityLeaf, IdentityPredicate};
use inventory_core::catalog::{Operation, Service, Target};

pub(in crate::runtime_catalog) fn validate(
    service: Service<'_>,
    operation: Operation<'_>,
    item: Target,
    identity: &Identity,
) -> Result<(), RuntimeCatalogCompileError> {
    match identity {
        Identity::MapArn {
            path,
            resource_kind,
            request_url,
            ..
        } => {
            text_map(service, item, path.as_str())?;
            if matches!(resource_kind, crate::overlay::ArnResourceKind::SqsQueue)
                != request_url.is_some()
            {
                return Err(RuntimeCatalogCompileError::Path {
                    path: path.as_str().to_owned(),
                });
            }
            request_url.as_ref().map_or(Ok(()), |path| {
                text(service, target(service, operation.input()?, path.as_str())?)
            })
        }
        Identity::CorrelatedKey(fields) => validate_fields(service, operation, item, fields),
        Identity::Choice(choice) => {
            choice
                .validate()
                .map_err(|_| RuntimeCatalogCompileError::Path {
                    path: "identity choice validation".to_owned(),
                })?;
            match choice {
                IdentityChoice::PreferArn { path, fallback, .. } => {
                    text(service, target(service, item, path.as_str())?)?;
                    leaf(service, operation, item, fallback)
                }
                IdentityChoice::Discriminated(branches) => branches.iter().try_for_each(|branch| {
                    match &branch.predicate {
                        IdentityPredicate::Present(path) => {
                            scalar(service, target(service, item, path.as_str())?)?
                        }
                        IdentityPredicate::TextEquals { path, .. } => {
                            text(service, target(service, item, path.as_str())?)?
                        }
                        IdentityPredicate::RequestTextEquals { path, .. } => {
                            text(service, target(service, operation.input()?, path.as_str())?)?
                        }
                    }
                    leaf(service, operation, item, &branch.identity)
                }),
            }
        }
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "identity".to_owned(),
        }),
    }
}
fn leaf(
    service: Service<'_>,
    operation: Operation<'_>,
    item: Target,
    identity: &IdentityLeaf,
) -> Result<(), RuntimeCatalogCompileError> {
    match identity {
        IdentityLeaf::Arn(path) => text(service, target(service, item, path.as_str())?),
        IdentityLeaf::Template(template) => super::super::plans::identity::fields(template)
            .try_for_each(|path| scalar(service, target(service, item, path)?)),
        IdentityLeaf::Key(paths) => paths
            .iter()
            .try_for_each(|path| scalar(service, target(service, item, path.as_str())?)),
        IdentityLeaf::Fields(fields) => validate_fields(service, operation, item, fields),
    }
}
fn validate_fields(
    service: Service<'_>,
    operation: Operation<'_>,
    item: Target,
    fields: &[IdentityField],
) -> Result<(), RuntimeCatalogCompileError> {
    if fields.is_empty() || fields.len() > 64 {
        return Err(RuntimeCatalogCompileError::Path {
            path: "identity fields".to_owned(),
        });
    }
    fields
        .iter()
        .try_for_each(|field| validate_single_field(service, operation, item, field))
}

fn validate_single_field(
    service: Service<'_>,
    operation: Operation<'_>,
    item: Target,
    field: &IdentityField,
) -> Result<(), RuntimeCatalogCompileError> {
    match field {
        IdentityField::Selected(path) => scalar(service, target(service, item, path.as_str())?),
        IdentityField::SelectedDigest(path) => {
            digest_target(service, target(service, item, path.as_str())?)
        }
        IdentityField::Request(path) => {
            scalar(service, target(service, operation.input()?, path.as_str())?)
        }
        IdentityField::SelectedArn(path) => text(service, target(service, item, path.as_str())?),
        IdentityField::RequestArn(path) => {
            text(service, target(service, operation.input()?, path.as_str())?)
        }
        IdentityField::SelectedInteger {
            path,
            minimum,
            maximum,
        } => integral(
            service,
            target(service, item, path.as_str())?,
            *minimum,
            *maximum,
        ),
        IdentityField::RequestInteger {
            path,
            minimum,
            maximum,
        } => integral(
            service,
            target(service, operation.input()?, path.as_str())?,
            *minimum,
            *maximum,
        ),
        IdentityField::SelectedStringList {
            path,
            max_items,
            max_item_bytes,
        } => validate_string_list(service, item, path, *max_items, *max_item_bytes),
        IdentityField::MapValue { path, .. } => text_map(service, item, path.as_str()),
        IdentityField::Context { .. } => Ok(()),
    }
}

fn validate_string_list(
    service: Service<'_>,
    item: Target,
    path: &crate::overlay::MemberPath,
    max_items: u32,
    max_item_bytes: u32,
) -> Result<(), RuntimeCatalogCompileError> {
    if !(1..=64).contains(&max_items) || !(1..=4096).contains(&max_item_bytes) {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        });
    }
    let Target::Shape(index) = target(service, item, path.as_str())? else {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        });
    };
    let shape = service
        .shape(index)?
        .ok_or(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        })?;
    if shape.kind()? != "list" {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        });
    }
    text(
        service,
        shape.element()?.ok_or(RuntimeCatalogCompileError::Path {
            path: path.as_str().to_owned(),
        })?,
    )
}
fn target(
    service: Service<'_>,
    root: Target,
    path: &str,
) -> Result<Target, RuntimeCatalogCompileError> {
    if path == "$" {
        Ok(root)
    } else {
        model_path::resolve(service, root, path).map(|(_, target)| target)
    }
}
fn text_map(
    service: Service<'_>,
    root: Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    let Target::Shape(index) = target(service, root, path)? else {
        return Err(RuntimeCatalogCompileError::Path {
            path: format!("text_map target {path}"),
        });
    };
    let shape = service
        .shape(index)?
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: format!("text_map shape {path}"),
        })?;
    if shape.kind()? != "map" {
        return Err(RuntimeCatalogCompileError::Path {
            path: format!("text_map kind {path}"),
        });
    }
    text(
        service,
        shape
            .map_key()?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: format!("text_map key {path}"),
            })?,
    )?;
    text(
        service,
        shape
            .map_value()?
            .ok_or_else(|| RuntimeCatalogCompileError::Path {
                path: format!("text_map value {path}"),
            })?,
    )
}
fn text(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    let kind = model_path::kind(service, target)?;
    if matches!(kind, "string" | "enum") {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Path {
            path: format!("text kind {kind}"),
        })
    }
}
fn scalar(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    let kind = model_path::kind(service, target)?;
    if matches!(
        kind,
        "string" | "enum" | "boolean" | "byte" | "short" | "integer" | "long"
    ) {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Path {
            path: format!("scalar kind {kind}"),
        })
    }
}

fn integral(
    service: Service<'_>,
    target: Target,
    minimum: i64,
    maximum: i64,
) -> Result<(), RuntimeCatalogCompileError> {
    let kind = model_path::kind(service, target)?;
    let (low, high) = match kind {
        "byte" => (i64::from(i8::MIN), i64::from(i8::MAX)),
        "short" => (i64::from(i16::MIN), i64::from(i16::MAX)),
        "integer" => (i64::from(i32::MIN), i64::from(i32::MAX)),
        "long" => (i64::MIN, i64::MAX),
        _ => {
            return Err(RuntimeCatalogCompileError::Path {
                path: format!("integral kind {kind}"),
            });
        }
    };
    if minimum <= maximum && minimum >= low && maximum <= high {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Path {
            path: "integral range".to_owned(),
        })
    }
}

fn digest_target(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogCompileError> {
    let kind = model_path::kind(service, target)?;
    if matches!(
        kind,
        "string"
            | "enum"
            | "intEnum"
            | "boolean"
            | "byte"
            | "short"
            | "integer"
            | "long"
            | "float"
            | "double"
            | "bigInteger"
            | "bigDecimal"
            | "blob"
            | "timestamp"
            | "structure"
            | "union"
            | "list"
            | "map"
    ) {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Path {
            path: format!("digest_target kind {kind}"),
        })
    }
}
