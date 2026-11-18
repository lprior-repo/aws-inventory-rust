mod projection;
use super::super::{RuntimeCatalogCompileError, ServiceIndex};
use super::{encode, identity, selected_path};
use crate::overlay::{
    ContextSource, Identity, IdentityChoice, IdentityField, IdentityLeaf, IdentityPredicate,
    SelectionKind,
};
use inventory_core::record::Value;
pub(super) use projection::{compile as context_projection, emission};

pub(super) fn item(
    path: &crate::overlay::MemberPath,
    kind: SelectionKind,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let tag = match kind {
        SelectionKind::MapEntries => 2,
        SelectionKind::MapKeys => 3,
        SelectionKind::MapValues => 4,
        SelectionKind::List => 5,
        SelectionKind::Structure => 6,
        SelectionKind::Scalar => 7,
    };
    encode::map([
        (1, Value::Int(tag)),
        (2, selected_path(services, path.as_str())?),
    ])
}

pub(super) fn identity(
    value: &Identity,
    service: &str,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        Identity::Choice(choice) => {
            choice
                .validate()
                .map_err(|_| RuntimeCatalogCompileError::Capability)?;
            match choice {
                IdentityChoice::PreferArn {
                    path,
                    fallback,
                    review,
                } => encode::map([
                    (1, Value::Int(3)),
                    (2, selected_path(services, path.as_str())?),
                    (3, leaf(fallback, service, services)?),
                    (4, Value::Text(review.clone())),
                ]),
                IdentityChoice::Discriminated(branches) => encode::map([
                    (1, Value::Int(4)),
                    (
                        2,
                        Value::List(
                            branches
                                .iter()
                                .map(|branch| {
                                    encode::map([
                                        (1, predicate(&branch.predicate, services)?),
                                        (2, leaf(&branch.identity, service, services)?),
                                    ])
                                })
                                .collect::<Result<_, _>>()?,
                        ),
                    ),
                ]),
            }
        }
        Identity::CorrelatedKey(fields) => fields_value(fields, service, services),
        Identity::MapArn {
            path,
            key,
            resource_kind,
            request_url,
        } => {
            let mut fields = vec![
                (1, Value::Int(6)),
                (2, selected_path(services, path.as_str())?),
                (3, Value::Text(key.clone())),
                (4, Value::Int(arn_kind(*resource_kind))),
            ];
            if let Some(path) = request_url {
                fields.push((5, selected_path(services, path.as_str())?));
            }
            encode::map(fields)
        }
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn leaf(
    value: &IdentityLeaf,
    service: &str,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        IdentityLeaf::Arn(path) => encode::map([
            (1, Value::Int(0)),
            (2, selected_path(services, path.as_str())?),
        ]),
        IdentityLeaf::Template(template) => identity::compile(template, services),
        IdentityLeaf::Key(paths) => encode::map([
            (1, Value::Int(2)),
            (
                2,
                Value::List(
                    paths
                        .iter()
                        .map(|path| selected_path(services, path.as_str()))
                        .collect::<Result<_, _>>()?,
                ),
            ),
        ]),
        IdentityLeaf::Fields(fields) => fields_value(fields, service, services),
    }
}

fn predicate(
    value: &IdentityPredicate,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        IdentityPredicate::Present(path) => encode::map([
            (1, Value::Int(0)),
            (2, selected_path(services, path.as_str())?),
        ]),
        IdentityPredicate::TextEquals { path, value } => encode::map([
            (1, Value::Int(1)),
            (2, selected_path(services, path.as_str())?),
            (3, Value::Text(value.clone())),
        ]),
        IdentityPredicate::RequestTextEquals { path, value } => encode::map([
            (1, Value::Int(2)),
            (2, selected_path(services, path.as_str())?),
            (3, Value::Text(value.clone())),
        ]),
    }
}

fn fields_value(
    fields: &[IdentityField],
    service: &str,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    if fields.is_empty() || fields.len() > 64 {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    encode::map([
        (1, Value::Int(5)),
        (
            2,
            Value::List(
                fields
                    .iter()
                    .map(|field| field_value(field, service, services))
                    .collect::<Result<_, _>>()?,
            ),
        ),
    ])
}

fn field_value(
    field: &IdentityField,
    service: &str,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match field {
        IdentityField::Selected(path)
        | IdentityField::SelectedDigest(path)
        | IdentityField::Request(path) => {
            let tag = match field {
                IdentityField::Selected(_) => 0,
                IdentityField::SelectedDigest(_) => 1,
                _ => 4,
            };
            encode::map([
                (1, Value::Int(tag)),
                (2, selected_path(services, path.as_str())?),
            ])
        }
        IdentityField::SelectedArn(path) | IdentityField::RequestArn(path) => {
            let tag = if matches!(field, IdentityField::SelectedArn(_)) {
                8
            } else {
                9
            };
            encode::map([
                (1, Value::Int(tag)),
                (2, selected_path(services, path.as_str())?),
            ])
        }
        IdentityField::SelectedInteger {
            path,
            minimum,
            maximum,
        }
        | IdentityField::RequestInteger {
            path,
            minimum,
            maximum,
        } => {
            if minimum > maximum {
                return Err(RuntimeCatalogCompileError::Capability);
            }
            let tag = if matches!(field, IdentityField::SelectedInteger { .. }) {
                5
            } else {
                6
            };
            encode::map([
                (1, Value::Int(tag)),
                (2, selected_path(services, path.as_str())?),
                (3, Value::Int(*minimum)),
                (4, Value::Int(*maximum)),
            ])
        }
        IdentityField::SelectedStringList {
            path,
            max_items,
            max_item_bytes,
        } => {
            if !(1..=64).contains(max_items) || !(1..=4096).contains(max_item_bytes) {
                return Err(RuntimeCatalogCompileError::Capability);
            }
            encode::map([
                (1, Value::Int(7)),
                (2, selected_path(services, path.as_str())?),
                (3, Value::Int(i64::from(*max_items))),
                (4, Value::Int(i64::from(*max_item_bytes))),
            ])
        }
        IdentityField::Context {
            operation,
            source,
            path,
        } => encode::map([
            (1, Value::Int(2)),
            (2, Value::Text(service.to_owned())),
            (3, Value::Text(operation.to_string())),
            (4, Value::Int(source_tag(*source))),
            (5, selected_path(services, path.as_str())?),
        ]),
        IdentityField::MapValue { path, key } => encode::map([
            (1, Value::Int(3)),
            (2, selected_path(services, path.as_str())?),
            (3, Value::Text(key.clone())),
        ]),
    }
}

pub(super) fn source_tag(source: ContextSource) -> i64 {
    match source {
        ContextSource::SelectedItem => 5,
        ContextSource::Response => 6,
        ContextSource::Request => 7,
    }
}

pub(super) fn arn_kind(kind: crate::overlay::ArnResourceKind) -> i64 {
    match kind {
        crate::overlay::ArnResourceKind::SnsSubscription => 0,
        crate::overlay::ArnResourceKind::SqsQueue => 1,
        crate::overlay::ArnResourceKind::SnsTopic => 2,
    }
}
