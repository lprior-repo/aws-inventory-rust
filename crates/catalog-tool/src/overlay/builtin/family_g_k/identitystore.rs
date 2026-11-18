//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/identitystore/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{
    MemberPath, MissingSelected, NameError, RowPredicate, RowValueType, ServiceOverlay,
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut groups = policy::query(
        policy::bind(
            policy::enumerate(
                "Groups",
                policy::arn("GroupArn")?,
                Some(("NextToken", "NextToken")),
            )?,
            "IdentityStoreId",
            "ListIdentityStores",
            "IdentityStoreId",
        )?,
        &["GroupArn", "GroupId", "IdentityStoreId"],
    )?;
    groups
        .row_policy
        .assertions
        .push(RowPredicate::SelectedRequestEquals {
            selected: MemberPath::selected("IdentityStoreId")?,
            request: MemberPath::selected("IdentityStoreId")?,
            value_type: RowValueType::String,
            missing: MissingSelected::AllowAbsent,
        });
    let approved = vec![
        (
            "ListIdentityStores",
            policy::query(
                policy::enumerate(
                    "IdentityStores",
                    policy::arn("IdentityStoreArn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["IdentityStoreArn"],
            )?,
        ),
        (
            "DescribeIdentityStore",
            policy::query(
                policy::bind(
                    policy::enumerate("$", policy::arn("IdentityStoreArn")?, None)?,
                    "IdentityStoreId",
                    "ListIdentityStores",
                    "IdentityStoreId",
                )?,
                &["IdentityStoreArn"],
            )?,
        ),
        ("ListGroups", groups),
    ];
    policy::finish(
        "identitystore",
        approved,
        DEFERRED,
        "Identity-store containers, network metadata and store-bound group metadata are approved. Group descriptions, external IDs and audit identifiers are configuration metadata, not user credentials or contact profiles. User profiles remain content-denied; membership traversal is separately reviewed.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeGroup",
    "DescribeGroupMembership",
    "GetGroupId",
    "GetGroupMembershipId",
    "GetUserId",
    "IsMemberInGroups",
    "ListGroupMemberships",
    "ListGroupMembershipsForMember",
];

const MUTATING: &[&str] = &[
    "CreateGroup",
    "CreateGroupMembership",
    "CreateUser",
    "DeleteGroup",
    "DeleteGroupMembership",
    "DeleteUser",
    "UpdateGroup",
    "UpdateIdentityStore",
    "UpdateUser",
];

const CONTENT: &[&str] = &["DescribeUser", "ListUsers"];
