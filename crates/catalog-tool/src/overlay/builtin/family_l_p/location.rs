use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AssociateTrackerConsumer",
            "BatchDeleteDevicePositionHistory",
            "BatchDeleteGeofence",
            "BatchEvaluateGeofences",
            "BatchGetDevicePosition",
            "BatchPutGeofence",
            "BatchUpdateDevicePosition",
            "CancelJob",
            "CreateGeofenceCollection",
            "CreateKey",
            "CreateMap",
            "CreatePlaceIndex",
            "CreateRouteCalculator",
            "CreateTracker",
            "DeleteGeofenceCollection",
            "DeleteKey",
            "DeleteMap",
            "DeletePlaceIndex",
            "DeleteRouteCalculator",
            "DeleteTracker",
            "DisassociateTrackerConsumer",
            "PutGeofence",
            "StartJob",
            "TagResource",
            "UntagResource",
            "UpdateGeofenceCollection",
            "UpdateKey",
            "UpdateMap",
            "UpdatePlaceIndex",
            "UpdateRouteCalculator",
            "UpdateTracker",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "CalculateRoute",
            "CalculateRouteMatrix",
            "ForecastGeofenceEvents",
            "GetPlace",
            "SearchPlaceIndexForPosition",
            "SearchPlaceIndexForSuggestions",
            "SearchPlaceIndexForText",
        ],
        class: OverlayClass::Deny(DenyReason::Paid),
        reason: None,
    },
    Group {
        operations: &["DescribeGeofenceCollection"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (CollectionName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeKey"],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["DescribeMap"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (MapName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribePlaceIndex"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (IndexName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeRouteCalculator"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (CalculatorName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeTracker", "ListTrackerConsumers"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TrackerName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetDevicePosition",
            "GetDevicePositionHistory",
            "GetGeofence",
            "GetMapGlyphs",
            "GetMapSprites",
            "GetMapStyleDescriptor",
            "GetMapTile",
            "ListDevicePositions",
            "ListGeofences",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["GetJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (JobId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListJobs", "ListKeys"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["VerifyDevicePosition"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TrackerName, DeviceState); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListGeofenceCollections",
        items: "Entries",
        id: Id::Key(&["CollectionName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["CollectionName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListMaps",
        items: "Entries",
        id: Id::Key(&["MapName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["MapName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListPlaceIndexes",
        items: "Entries",
        id: Id::Key(&["IndexName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["IndexName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListRouteCalculators",
        items: "Entries",
        id: Id::Key(&["CalculatorName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["CalculatorName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListTrackers",
        items: "Entries",
        id: Id::Key(&["TrackerName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["TrackerName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("location", GROUPS, RESOURCES)
}
