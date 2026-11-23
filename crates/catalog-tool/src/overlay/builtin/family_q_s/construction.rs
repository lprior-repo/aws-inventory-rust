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
    Correlated(&'static str, &'static [&'static str]),
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
    Structure(&'static str, &'static [(&'static str, &'static str)]),
    Filter(
        &'static str,
        &'static str,
        &'static str,
        &'static [&'static str],
    ),
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
        identity: Some(identity(&spec.identity)?),
        paginator: Some(paginator(spec.token)?),
        queryable: queryable(spec)?,
        tags: spec.tags.map(path).transpose()?,
        ..empty()
    };
    Ok((
        spec.operation,
        spec.input.iter().try_fold(overlay, bind_input)?,
    ))
}

fn identity(spec: &ResourceIdentity) -> Result<Identity, NameError> {
    Ok(match spec {
        ResourceIdentity::Arn(member) => Identity::Arn(path(member)?),
        ResourceIdentity::Key(members) => Identity::Key(
            members
                .iter()
                .map(|member| path(member))
                .collect::<Result<_, _>>()?,
        ),
        ResourceIdentity::Correlated(request, selected) => Identity::CorrelatedKey(
            std::iter::once(path(request).map(IdentityField::Request))
                .chain(
                    selected
                        .iter()
                        .map(|member| path(member).map(IdentityField::Selected)),
                )
                .collect::<Result<_, _>>()?,
        ),
        ResourceIdentity::Template(template, _) => Identity::Template((*template).to_owned()),
    })
}

fn queryable(spec: &Inventory) -> Result<Vec<crate::overlay::MemberPath>, NameError> {
    let members = match &spec.identity {
        ResourceIdentity::Arn(member) => std::slice::from_ref(member),
        ResourceIdentity::Correlated(_, members)
        | ResourceIdentity::Key(members)
        | ResourceIdentity::Template(_, members) => *members,
    };
    members
        .iter()
        .chain(spec.queryable)
        .map(|member| path(member))
        .collect()
}

fn paginator(token: Option<(&str, &str)>) -> Result<Paginator, NameError> {
    Ok(match token {
        Some((input, output)) => Paginator::Token(TokenPair {
            input: path(input)?,
            output: path(output)?,
            optional: false,
        }),
        None => Paginator::None,
    })
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
        _ => {
            if let Some((member, value)) = fixed_input(input)? {
                overlay.fixed.insert(path(member)?, value);
            }
        }
    }
    Ok(overlay)
}

fn fixed_input(input: &Input) -> Result<Option<(&str, FixedValue)>, NameError> {
    let value = match input {
        Input::String(member, value) => (*member, FixedValue::String((*value).to_owned())),
        Input::Boolean(member, value) => (*member, FixedValue::Boolean(*value)),
        Input::Strings(member, values) => (
            *member,
            FixedValue::Strings(values.iter().map(|value| (*value).to_owned()).collect()),
        ),
        Input::Timestamp(member, seconds, nanos) => (
            *member,
            FixedValue::Timestamp(Time::new(*seconds, *nanos).map_err(|_| NameError {
                value: "timestamp nanoseconds out of range".to_owned(),
            })?),
        ),
        Input::Structure(member, values) => (
            *member,
            FixedValue::structure(
                values
                    .iter()
                    .map(|(name, value)| (*name, FixedValue::String((*value).to_owned()))),
            )?,
        ),
        Input::Filter(member, key, value, values) => (*member, filter(key, value, values)?),
        Input::Account(_) | Input::Parent(..) => return Ok(None),
    };
    Ok(Some(value))
}

fn filter(key: &str, value: &str, values: &[&str]) -> Result<FixedValue, NameError> {
    Ok(FixedValue::list([FixedValue::structure([
        (key, FixedValue::String(value.to_owned())),
        (
            "Values",
            FixedValue::Strings(values.iter().map(|value| (*value).to_owned()).collect()),
        ),
    ])?]))
}
