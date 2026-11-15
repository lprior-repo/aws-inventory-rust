use crate::overlay::{NameError, OperationOverlay, Identity, ItemSelector, OverlayClass, Paginator, Scope};
use super::super::{empty, path};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetCallerIdentity", OperationOverlay {
        scope: Some(Scope::Account),
        class: Some(OverlayClass::Enumerate),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("Account")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetAccessKeyInfo", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Output schema, stable identity, required inputs, and relationship fields are not yet individually reviewed.".to_owned()),
        ..empty()
    }),
("GetDelegatedAccessToken", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetFederationToken", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetWebIdentityToken", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    })
    ])
}
