use crate::overlay::{NameError, OperationOverlay, OverlayClass};
use super::super::{empty};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListOrganizationalUnitsForParent", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required ParentId must start at ListRoots.Id and recursively consume returned OrganizationalUnits.Id. The schema permits one nonrecursive source and rejects self-bindings/cycles; a roots-only grant silently misses nested OUs. Need bounded recursive union traversal, duplicate/cycle detection and preserved ParentId for hierarchy edges. Items provide Arn identity.".to_owned()),
        ..empty()
    }),
("ListParents", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required ChildId needs the union of all ListAccounts.Id and recursively discovered OU Id values. Parents items contain only parent Id and Type, not the queried child. Parent identity alone merges distinct memberships. Need source unions plus request-child identity/edge context and single-parent validation; binding accounts only would omit OU-parent relationships.".to_owned()),
        ..empty()
    }),
("ListPolicies", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required Filter is a scalar PolicyType with 15 modeled values. Need reviewed scalar enum fan-out; a fixed string array is not a valid Filter. ListRoots.PolicyTypes exposes enabled root types, not an exhaustive stored-policy source for every modeled type. Policies items provide Arn identity once all types can be enumerated.".to_owned()),
        ..empty()
    }),
("ListPoliciesForTarget", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required TargetId needs roots, accounts and recursively discovered OUs; required Filter needs all 15 scalar PolicyType values. Policies items omit TargetId, so policy attachment edges require request-target context. Need source unions, complete OU traversal, scalar enum fan-out and contextual relationships; no one-target or one-type narrowing.".to_owned()),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ResourceId documentation covers accounts, roots, OUs and policies; modeled TaggableResourceId additionally accepts resource-policy and responsibility-transfer IDs. Complete source union is unavailable. Tags items contain only Key and Value, so identity and resource-tag attachment must preserve the queried ResourceId; a Key-only identity collides across resources. NextToken pagination has no MaxResults member.".to_owned()),
        ..empty()
    }),
("ListTargetsForPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required PolicyId depends on exhaustive ListPolicies discovery, currently blocked by scalar enum fan-out. Targets items provide target Arn but omit PolicyId; policy-target attachment edges require preserved request-policy context rather than merging attachments by target ARN.".to_owned()),
        ..empty()
    }),
("DescribeHandshake", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListHandshakesForAccount", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListHandshakesForOrganization", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("AcceptHandshake", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("AttachPolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CancelHandshake", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CloseAccount", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateAccount", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateGovCloudAccount", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateOrganization", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateOrganizationalUnit", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreatePolicy", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeclineHandshake", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteOrganization", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
