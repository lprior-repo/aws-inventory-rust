use super::super::service;
use super::{defer, deny, verifiedpermissions_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "verifiedpermissions",
        vec![
            (
                "BatchGetPolicy",
                defer(
                    "Missing executable inventory binding for requests: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "BatchIsAuthorized",
                defer(
                    "Missing executable inventory binding for policyStoreId, requests: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "BatchIsAuthorizedWithToken",
                defer(
                    "Missing executable inventory binding for policyStoreId, requests: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("CreateIdentitySource", deny(DenyReason::Mutating)),
            ("CreatePolicy", deny(DenyReason::Mutating)),
            ("CreatePolicyStore", deny(DenyReason::Mutating)),
            ("CreatePolicyStoreAlias", deny(DenyReason::Mutating)),
            ("CreatePolicyTemplate", deny(DenyReason::Mutating)),
            ("DeleteIdentitySource", deny(DenyReason::Mutating)),
            ("DeletePolicy", deny(DenyReason::Mutating)),
            ("DeletePolicyStore", deny(DenyReason::Mutating)),
            ("DeletePolicyStoreAlias", deny(DenyReason::Mutating)),
            ("DeletePolicyTemplate", deny(DenyReason::Mutating)),
            (
                "GetIdentitySource",
                verifiedpermissions_plans_1::get_identity_source()?,
            ),
            (
                "GetPolicy",
                defer(
                    "Missing executable inventory binding for policyStoreId, policyId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetPolicyStore",
                verifiedpermissions_plans_1::get_policy_store()?,
            ),
            (
                "GetPolicyStoreAlias",
                verifiedpermissions_plans_1::get_policy_store_alias()?,
            ),
            (
                "GetPolicyTemplate",
                defer(
                    "Missing executable inventory binding for policyStoreId, policyTemplateId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetSchema",
                defer(
                    "Missing executable inventory binding for policyStoreId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "IsAuthorized",
                defer(
                    "Missing executable inventory binding for policyStoreId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "IsAuthorizedWithToken",
                defer(
                    "Missing executable inventory binding for policyStoreId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListIdentitySources",
                verifiedpermissions_plans_1::list_identity_sources()?,
            ),
            (
                "ListPolicies",
                verifiedpermissions_plans_1::list_policies()?,
            ),
            (
                "ListPolicyStoreAliases",
                verifiedpermissions_plans_1::list_policy_store_aliases()?,
            ),
            (
                "ListPolicyStores",
                verifiedpermissions_plans_1::list_policy_stores()?,
            ),
            (
                "ListPolicyTemplates",
                verifiedpermissions_plans_1::list_policy_templates()?,
            ),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            ("PutSchema", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateIdentitySource", deny(DenyReason::Mutating)),
            ("UpdatePolicy", deny(DenyReason::Mutating)),
            ("UpdatePolicyStore", deny(DenyReason::Mutating)),
            ("UpdatePolicyTemplate", deny(DenyReason::Mutating)),
        ],
    )
}
