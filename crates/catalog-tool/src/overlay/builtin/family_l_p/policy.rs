use super::super::{empty, name, path, service};
use crate::overlay::{
    Binding, FixedValue, Identity, IdentityChoice, IdentityField, IdentityLeaf, ItemSelector,
    NameError, OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};

pub(super) enum Id {
    Arn(&'static str),
    Key(&'static [&'static str]),
    PreferArn(&'static str, &'static str, &'static str),
    Correlated(&'static [&'static str], &'static [&'static str]),
}

pub(super) enum Input {
    String(&'static str, &'static str),
    Boolean(&'static str, bool),
    Filter(&'static str, &'static str, &'static [&'static str]),
}

pub(super) struct Group {
    pub operations: &'static [&'static str],
    pub class: OverlayClass,
    pub reason: Option<&'static str>,
}

pub(super) struct Resource {
    pub operation: &'static str,
    pub items: &'static str,
    pub id: Id,
    pub scope: Scope,
    pub tokens: Option<(&'static str, &'static str)>,
    pub query: &'static [&'static str],
    pub tags: Option<&'static str>,
    pub bindings: &'static [(&'static str, &'static str, &'static str)],
    pub fixed: &'static [Input],
}

pub(super) fn compile(
    service_name: &str,
    groups: &[Group],
    resources: &[Resource],
) -> Result<ServiceOverlay, NameError> {
    let blocked = groups.iter().flat_map(|group| {
        group.operations.iter().map(|operation| {
            let policy = OperationOverlay {
                class: Some(group.class),
                defer_reason: group.reason.map(str::to_owned),
                ..empty()
            };
            Ok((*operation, policy))
        })
    });
    let allowed = resources
        .iter()
        .map(|resource| inventory(resource).map(|policy| (resource.operation, policy)));
    service(
        service_name,
        blocked
            .chain(allowed)
            .collect::<Result<Vec<_>, NameError>>()?,
    )
}

fn inventory(resource: &Resource) -> Result<OperationOverlay, NameError> {
    let identity = match resource.id {
        Id::Arn(member) => Identity::Arn(path(member)?),
        Id::PreferArn(member, template, review) => Identity::Choice(IdentityChoice::PreferArn {
            path: path(member)?,
            fallback: IdentityLeaf::Template(template.to_owned()),
            review: review.to_owned(),
        }),
        Id::Key(members) => Identity::Key(
            members
                .iter()
                .map(|member| path(member))
                .collect::<Result<_, _>>()?,
        ),
        Id::Correlated(request, selected) => Identity::CorrelatedKey(
            request
                .iter()
                .map(|member| Ok(IdentityField::Request(path(member)?)))
                .chain(
                    selected
                        .iter()
                        .map(|member| Ok(IdentityField::Selected(path(member)?))),
                )
                .collect::<Result<_, NameError>>()?,
        ),
    };
    let paginator = match resource.tokens {
        Some((input, output)) => Paginator::Token(TokenPair {
            input: path(input)?,
            output: path(output)?,
            optional: false,
        }),
        None => Paginator::None,
    };
    Ok(OperationOverlay {
        class: Some(if resource.bindings.is_empty() {
            OverlayClass::Enumerate
        } else {
            OverlayClass::Expand
        }),
        scope: Some(resource.scope),
        items: Some(ItemSelector::parse(resource.items)?),
        identity: Some(identity),
        paginator: Some(paginator),
        queryable: resource
            .query
            .iter()
            .map(|member| path(member))
            .collect::<Result<_, _>>()?,
        tags: resource.tags.map(path).transpose()?,
        bind: bindings(resource.bindings)?,
        fixed: fixed(resource.fixed)?,
        ..empty()
    })
}

fn bindings(
    values: &[(&str, &str, &str)],
) -> Result<std::collections::BTreeMap<crate::overlay::MemberPath, Binding>, NameError> {
    values
        .iter()
        .map(|(input, operation, member)| {
            Ok((
                path(input)?,
                Binding::From {
                    operation: name(operation)?,
                    path: path(member)?,
                },
            ))
        })
        .collect()
}

fn fixed(
    values: &[Input],
) -> Result<std::collections::BTreeMap<crate::overlay::MemberPath, FixedValue>, NameError> {
    values
        .iter()
        .map(|value| match value {
            Input::String(member, value) => {
                Ok((path(member)?, FixedValue::String((*value).to_owned())))
            }
            Input::Boolean(member, value) => Ok((path(member)?, FixedValue::Boolean(*value))),
            Input::Filter(member, name, values) => Ok((
                path(member)?,
                FixedValue::list([FixedValue::structure([
                    ("Name", FixedValue::String((*name).to_owned())),
                    (
                        "Values",
                        FixedValue::Strings(
                            values.iter().map(|value| (*value).to_owned()).collect(),
                        ),
                    ),
                ])?]),
            )),
        })
        .collect()
}
