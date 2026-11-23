use super::super::{empty, name, path, service};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, IdentityField, ItemSelector, NameError, OperationOverlay,
    OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};
use inventory_core::record::Time;

pub(super) enum ResourceIdentity {
    Arn(&'static str),
    Key(&'static [&'static str]),
    Correlated(&'static str, &'static str),
    /// ARN template plus the item members it reads; aligns services that list one resource
    /// store under different member names.
    Template(&'static str, &'static [&'static str]),
}

pub(super) enum Input {
    Account(&'static str),
    Parent(&'static str, &'static str, &'static str),
    String(&'static str, &'static str),
    Timestamp(&'static str, i64, u32),
    Boolean(&'static str, bool),
    Strings(&'static str, &'static [&'static str]),
}

pub(super) struct Inventory {
    pub operation: &'static str,
    pub items: &'static str,
    pub identity: ResourceIdentity,
    pub token: Option<(&'static str, &'static str)>,
    pub scope: Scope,
    pub input: &'static [Input],
    pub tags: Option<&'static str>,
    pub queryable: &'static [&'static str],
}

pub(super) enum Decision {
    Deferred(&'static str),
    Denied(DenyReason),
}

pub(super) fn reviewed(
    key: &str,
    inventory: &[&[Inventory]],
    decisions: &[&[(&'static str, Decision)]],
) -> Result<ServiceOverlay, NameError> {
    let calls = inventory.iter().flat_map(|rows| rows.iter()).map(operation);
    let blocked = decisions
        .iter()
        .flat_map(|rows| rows.iter())
        .map(|(name, decision)| {
            let overlay = match decision {
                Decision::Deferred(reason) => OperationOverlay {
                    class: Some(OverlayClass::Defer),
                    defer_reason: Some((*reason).to_owned()),
                    ..empty()
                },
                Decision::Denied(reason) => OperationOverlay {
                    class: Some(OverlayClass::Deny(*reason)),
                    ..empty()
                },
            };
            Ok((*name, overlay))
        });
    service(
        key,
        calls
            .chain(blocked)
            .collect::<Result<Vec<_>, NameError>>()?,
    )
}

fn operation(spec: &Inventory) -> Result<(&'static str, OperationOverlay), NameError> {
    let identity = match &spec.identity {
        ResourceIdentity::Arn(member) => Identity::Arn(path(member)?),
        ResourceIdentity::Key(members) => Identity::Key(
            members
                .iter()
                .map(|member| path(member))
                .collect::<Result<_, _>>()?,
        ),
        ResourceIdentity::Correlated(request, selected) => Identity::CorrelatedKey(vec![
            IdentityField::Request(path(request)?),
            IdentityField::Selected(path(selected)?),
        ]),
        ResourceIdentity::Template(template, _) => Identity::Template((*template).to_owned()),
    };
    let mut queryable = match &spec.identity {
        ResourceIdentity::Arn(member) => vec![path(member)?],
        ResourceIdentity::Correlated(_, selected) => vec![path(selected)?],
        ResourceIdentity::Key(members) | ResourceIdentity::Template(_, members) => members
            .iter()
            .map(|member| path(member))
            .collect::<Result<_, _>>()?,
    };
    for member in spec.queryable {
        queryable.push(path(member)?);
    }
    let paginator = match spec.token {
        Some((input, output)) => Paginator::Token(TokenPair {
            input: path(input)?,
            output: path(output)?,
            optional: false,
        }),
        None => Paginator::None,
    };
    let overlay = OperationOverlay {
        class: Some(
            if spec
                .input
                .iter()
                .any(|input| matches!(input, Input::Parent(..)))
            {
                OverlayClass::Expand
            } else {
                OverlayClass::Enumerate
            },
        ),
        scope: Some(spec.scope),
        items: Some(ItemSelector::parse(spec.items)?),
        identity: Some(identity),
        paginator: Some(paginator),
        queryable,
        tags: spec.tags.map(path).transpose()?,
        ..empty()
    };
    Ok((
        spec.operation,
        spec.input.iter().try_fold(overlay, bind_input)?,
    ))
}

fn bind_input(mut overlay: OperationOverlay, input: &Input) -> Result<OperationOverlay, NameError> {
    match input {
        Input::Account(member) => {
            overlay.bind.insert(path(member)?, Binding::Account);
        }
        Input::Parent(member, operation, source) => {
            overlay.bind.insert(
                path(member)?,
                Binding::From {
                    operation: name(operation)?,
                    path: path(source)?,
                },
            );
        }
        Input::String(member, value) => {
            overlay
                .fixed
                .insert(path(member)?, FixedValue::String((*value).to_owned()));
        }
        Input::Timestamp(member, seconds, nanos) => {
            let time = Time::new(*seconds, *nanos).map_err(|_| NameError {
                value: "timestamp nanoseconds out of range".to_owned(),
            })?;
            overlay
                .fixed
                .insert(path(member)?, FixedValue::Timestamp(time));
        }
        Input::Boolean(member, value) => {
            overlay
                .fixed
                .insert(path(member)?, FixedValue::Boolean(*value));
        }
        Input::Strings(member, values) => {
            overlay.fixed.insert(
                path(member)?,
                FixedValue::Strings(values.iter().map(|value| (*value).to_owned()).collect()),
            );
        }
    }
    Ok(overlay)
}
