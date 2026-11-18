use super::super::empty;
pub(super) use super::super::{name, path, service};
pub(super) use crate::classes::DenyReason;
pub(super) use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, ServiceOverlay, TokenPair,
};

pub(super) fn read(
    items: &str,
    identity: Identity,
    scope: Scope,
    paginator: Paginator,
) -> Result<OperationOverlay, NameError> {
    Ok(OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(scope),
        items: Some(ItemSelector::parse(items)?),
        identity: Some(identity),
        paginator: Some(paginator),
        ..empty()
    })
}

pub(super) fn token(input: &str, output: &str) -> Result<Paginator, NameError> {
    Ok(Paginator::Token(TokenPair::new(
        path(input)?,
        path(output)?,
    )))
}

pub(super) fn deferred(reason: &str) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some(reason.to_owned()),
        ..empty()
    }
}

pub(super) fn denied(reason: DenyReason) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Deny(reason)),
        ..empty()
    }
}
