use super::{
    Identity, ItemSelector, MemberPath, OperationOverlay, Origin, OverlayClass, OverlayError,
    Paginator, Scope, ServiceOverlay, TokenPair,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format: i64,
    reviewed: String,
    reviewer: String,
    #[serde(default)]
    operation: BTreeMap<String, Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    class: String,
    defer_reason: Option<String>,
    scope: Option<String>,
    items: Option<String>,
    identity: Option<IdentityEntry>,
    paginator: Option<PaginatorEntry>,
    #[serde(default)]
    bind: BTreeMap<String, BindingEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityEntry {
    arn: Option<String>,
    key: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PaginatorEntry {
    kind: String,
    input: Option<String>,
    output: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingEntry {
    from: Option<String>,
    path: Option<String>,
    account: Option<bool>,
    region: Option<bool>,
}

impl BindingEntry {
    fn binding(&self) -> Result<super::Binding, OverlayError> {
        match (&self.from, &self.path, self.account, self.region) {
            (Some(operation), Some(path), None, None) => Ok(super::Binding::From {
                operation: super::OperationName::new(operation)?,
                path: MemberPath::selected(path)?,
            }),
            (None, None, Some(true), None) => Ok(super::Binding::Account),
            (None, None, None, Some(true)) => Ok(super::Binding::Region),
            _ => Err(invalid("unsupported generated binding")),
        }
    }
}

impl ServiceOverlay {
    pub fn parse(service: &str, text: &str, origin: Origin) -> Result<Self, OverlayError> {
        if text.len() > super::MAX_FILE_BYTES || service.len() > 128 {
            return Err(invalid("file or service limit"));
        }
        let document: Document = toml::from_str(text)?;
        validate_header(&document, origin)?;
        let entries = document
            .operation
            .iter()
            .map(|(name, entry)| entry.overlay().map(|entry| (name.as_str(), entry)))
            .collect::<Result<Vec<_>, _>>()?;
        let mut policy = Self::new(service, entries)?;
        apply_metadata(&mut policy, &document, origin);
        Ok(policy)
    }
}

fn apply_metadata(policy: &mut ServiceOverlay, document: &Document, origin: Origin) {
    policy.reviewed.clone_from(&document.reviewed);
    policy.reviewer.clone_from(&document.reviewer);
    policy.origin = policy
        .operations
        .keys()
        .cloned()
        .map(|name| (name, origin))
        .collect();
}

fn validate_header(document: &Document, origin: Origin) -> Result<(), OverlayError> {
    if document.format != 1 || document.operation.len() > super::MAX_OPERATIONS {
        return Err(invalid("format or operation limit"));
    }
    if (document.reviewer == "generated") != (origin == Origin::Generated) {
        return Err(invalid("reviewer does not match policy origin"));
    }
    for value in [&document.reviewer, &document.reviewed] {
        if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
            return Err(invalid("invalid review metadata"));
        }
    }
    Ok(())
}

impl Entry {
    fn overlay(&self) -> Result<OperationOverlay, OverlayError> {
        Ok(OperationOverlay {
            class: Some(class(&self.class)?),
            defer_reason: self.defer_reason.clone(),
            bind: self
                .bind
                .iter()
                .map(|(path, binding)| Ok((MemberPath::new(path)?, binding.binding()?)))
                .collect::<Result<_, OverlayError>>()?,
            scope: self.scope.as_deref().map(scope).transpose()?,
            items: self.items.as_deref().map(ItemSelector::parse).transpose()?,
            identity: self
                .identity
                .as_ref()
                .map(IdentityEntry::identity)
                .transpose()?,
            paginator: self
                .paginator
                .as_ref()
                .map(PaginatorEntry::paginator)
                .transpose()?,
            ..OperationOverlay::default()
        })
    }
}

fn class(value: &str) -> Result<OverlayClass, OverlayError> {
    match value {
        "enumerate" => Ok(OverlayClass::Enumerate),
        "expand" => Ok(OverlayClass::Expand),
        "defer" => Ok(OverlayClass::Defer),
        _ => Err(invalid("unsupported generated class")),
    }
}

fn scope(value: &str) -> Result<Scope, OverlayError> {
    match value {
        "region" => Ok(Scope::Region),
        "account" => Ok(Scope::Account),
        "home_region" => Ok(Scope::HomeRegion),
        _ => Err(invalid("unknown scope")),
    }
}

impl IdentityEntry {
    fn identity(&self) -> Result<Identity, OverlayError> {
        match (&self.arn, &self.key) {
            (Some(arn), None) => Ok(Identity::Arn(MemberPath::selected(arn)?)),
            (None, Some(keys)) if !keys.is_empty() && keys.len() <= 32 => Ok(Identity::Key(
                keys.iter()
                    .map(|key| MemberPath::selected(key))
                    .collect::<Result<_, _>>()?,
            )),
            _ => Err(invalid(
                "identity must have exactly one ARN or nonempty key",
            )),
        }
    }
}

impl PaginatorEntry {
    fn paginator(&self) -> Result<Paginator, OverlayError> {
        match (self.kind.as_str(), &self.input, &self.output) {
            ("none", None, None) => Ok(Paginator::None),
            ("token", Some(input), Some(output)) => Ok(Paginator::Token(TokenPair::new(
                MemberPath::new(input)?,
                MemberPath::new(output)?,
            ))),
            _ => Err(invalid("invalid generated paginator")),
        }
    }
}

fn invalid(reason: &str) -> OverlayError {
    OverlayError::Invalid(reason.to_owned())
}
