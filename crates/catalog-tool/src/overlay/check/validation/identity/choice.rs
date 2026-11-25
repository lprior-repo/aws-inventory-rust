mod typed_fields;
use crate::{
    overlay::{
        ContextSource, Identity, IdentityChoice, IdentityField, IdentityLeaf, IdentityPredicate,
    },
    plans::{
        Model,
        model::{ShapeRef, shape_type},
    },
};

pub(super) fn valid<'m>(
    operation: &str,
    model: &Model<'m>,
    item: ShapeRef<'m>,
    identity: &Identity,
) -> bool {
    match identity {
        Identity::MapArn {
            path,
            key,
            resource_kind,
            request_url,
        } => {
            !key.is_empty()
                && key.len() <= 4096
                && text_map(model, item, path.as_str())
                && (matches!(resource_kind, crate::overlay::ArnResourceKind::SqsQueue)
                    == request_url.is_some())
                && request_url.as_ref().is_none_or(|path| {
                    text(
                        model
                            .resolve_path(operation, "input", path.as_str())
                            .ok()
                            .flatten(),
                    )
                })
        }
        Identity::CorrelatedKey(fields) => valid_fields(operation, model, item, fields),
        Identity::Choice(choice) => {
            choice.validate().is_ok()
                && match choice {
                    IdentityChoice::PreferArn { path, fallback, .. } => {
                        text(selected(model, item, path.as_str()))
                            && leaf(operation, model, item, fallback)
                    }
                    IdentityChoice::Discriminated(branches) => branches.iter().all(|branch| {
                        let predicate = match &branch.predicate {
                            IdentityPredicate::Present(path) => {
                                scalar(selected(model, item, path.as_str()))
                            }
                            IdentityPredicate::TextEquals { path, .. } => {
                                text(selected(model, item, path.as_str()))
                            }
                            IdentityPredicate::RequestTextEquals { path, .. } => text(
                                model
                                    .resolve_path(operation, "input", path.as_str())
                                    .ok()
                                    .flatten(),
                            ),
                        };
                        predicate && leaf(operation, model, item, &branch.identity)
                    }),
                }
        }
        _ => false,
    }
}
fn leaf<'m>(operation: &str, model: &Model<'m>, item: ShapeRef<'m>, leaf: &IdentityLeaf) -> bool {
    match leaf {
        IdentityLeaf::Arn(path) => text(selected(model, item, path.as_str())),
        IdentityLeaf::Key(paths) => {
            !paths.is_empty()
                && paths.len() <= 8
                && paths
                    .iter()
                    .all(|path| scalar(selected(model, item, path.as_str())))
        }
        IdentityLeaf::Template(template) => super::template_paths(template)
            .iter()
            .all(|path| scalar(selected(model, item, path))),
        IdentityLeaf::Fields(fields) => valid_fields(operation, model, item, fields),
    }
}
fn valid_fields<'m>(
    operation: &str,
    model: &Model<'m>,
    item: ShapeRef<'m>,
    fields: &[IdentityField],
) -> bool {
    !fields.is_empty()
        && fields.len() <= 64
        && fields
            .iter()
            .all(|field| valid_field(operation, model, item, field))
}

fn valid_field<'m>(
    operation: &str,
    model: &Model<'m>,
    item: ShapeRef<'m>,
    field: &IdentityField,
) -> bool {
    match field {
        IdentityField::Selected(path) => scalar(selected(model, item, path.as_str())),
        IdentityField::SelectedDigest(path) => typed_fields::digest(if path.as_str() == "$" {
            Some(item)
        } else {
            selected(model, item, path.as_str())
        }),
        IdentityField::Request(path) => scalar(request(model, operation, path.as_str())),
        IdentityField::SelectedArn(path) => text(selected(model, item, path.as_str())),
        IdentityField::RequestArn(path) => text(request(model, operation, path.as_str())),
        IdentityField::SelectedInteger {
            path,
            minimum,
            maximum,
        } => typed_fields::integer(selected(model, item, path.as_str()), *minimum, *maximum),
        IdentityField::RequestInteger {
            path,
            minimum,
            maximum,
        } => typed_fields::integer(request(model, operation, path.as_str()), *minimum, *maximum),
        IdentityField::SelectedStringList {
            path,
            max_items,
            max_item_bytes,
        } => typed_fields::string_list(
            model,
            selected(model, item, path.as_str()),
            *max_items,
            *max_item_bytes,
        ),
        IdentityField::MapValue { path, key } => {
            !key.is_empty() && key.len() <= 4096 && text_map(model, item, path.as_str())
        }
        IdentityField::Context {
            operation,
            source,
            path,
        } => field_context_valid(operation, source, path, model),
    }
}

fn request<'m>(model: &Model<'m>, operation: &str, path: &str) -> Option<ShapeRef<'m>> {
    model.resolve_path(operation, "input", path).ok().flatten()
}

fn field_context_valid(
    operation: &crate::overlay::OperationName,
    source: &ContextSource,
    path: &crate::overlay::MemberPath,
    model: &Model<'_>,
) -> bool {
    model.operation_id(operation.as_str()).is_some()
        && match source {
            // Producer-relative selectors require the cross-plan dependency gate, not the consumer model root.
            ContextSource::SelectedItem => true,
            ContextSource::Response => scalar(
                model
                    .resolve_path(operation.as_str(), "output", path.as_str())
                    .ok()
                    .flatten(),
            ),
            ContextSource::Request => scalar(
                model
                    .resolve_path(operation.as_str(), "input", path.as_str())
                    .ok()
                    .flatten(),
            ),
        }
}
fn selected<'m>(model: &Model<'m>, item: ShapeRef<'m>, path: &str) -> Option<ShapeRef<'m>> {
    super::super::paths::resolve_selected(model, item, path)
}
fn scalar(shape: Option<ShapeRef<'_>>) -> bool {
    shape.is_some_and(|shape| {
        matches!(
            shape_type(shape.value),
            Some("string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean")
        )
    })
}
fn text(shape: Option<ShapeRef<'_>>) -> bool {
    shape.is_some_and(|shape| matches!(shape_type(shape.value), Some("string" | "enum")))
}
fn text_map(model: &Model<'_>, item: ShapeRef<'_>, path: &str) -> bool {
    selected(model, item, path).is_some_and(|shape| {
        shape_type(shape.value) == Some("map")
            && ["key", "value"].iter().all(|field| {
                text(
                    shape
                        .value
                        .get(field)
                        .and_then(|member| member.get("target"))
                        .and_then(serde_json::Value::as_str)
                        .and_then(|id| model.shape(id)),
                )
            })
    })
}
