use super::super::empty;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListInvalidationsForDistributionTenant", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Can obtain tenant Id from ListDistributionTenants, but InvalidationList.Items omit that tenant Id. Need request-context composite identity to distinguish invalidations scoped to different tenants.".to_owned()),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires Resource ARN across heterogeneous eligible CloudFront kinds. Tags output omits the resource ARN; need multi-source bindings and request-context identity/tag attachment, not independent tag resources.".to_owned()),
        ..empty()
    })
    ])
}
