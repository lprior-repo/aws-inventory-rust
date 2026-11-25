use super::super::service;
use super::{defer, deny};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "timestream-query",
        vec![
            ("CancelQuery", deny(DenyReason::Mutating)),
            ("CreateScheduledQuery", deny(DenyReason::Mutating)),
            ("DeleteScheduledQuery", deny(DenyReason::Mutating)),
            (
                "DescribeAccountSettings",
                defer(
                    "This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.",
                ),
            ),
            ("DescribeEndpoints", deny(DenyReason::ReferenceData)),
            ("DescribeScheduledQuery", deny(DenyReason::Content)),
            (
                "ExecuteScheduledQuery",
                defer(
                    "This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.",
                ),
            ),
            (
                "ListScheduledQueries",
                defer(
                    "This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.",
                ),
            ),
            (
                "ListTagsForResource",
                defer(
                    "This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.",
                ),
            ),
            (
                "PrepareQuery",
                defer(
                    "This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.",
                ),
            ),
            ("Query", deny(DenyReason::Content)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateAccountSettings", deny(DenyReason::Mutating)),
            ("UpdateScheduledQuery", deny(DenyReason::Mutating)),
        ],
    )
}
