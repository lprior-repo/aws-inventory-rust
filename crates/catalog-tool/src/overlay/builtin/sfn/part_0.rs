use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListActivities", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("activities")?),
        queryable: vec![path("name")?, path("creationDate")?],
        fixed: [(path("maxResults")?, FixedValue::Integer(1000))].into_iter().collect(),
        identity: Some(Identity::Arn(path("activityArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("DescribeActivity", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        queryable: vec![path("name")?, path("creationDate")?, path("encryptionConfiguration.type")?],
        bind: [(path("activityArn")?, Binding::From { operation: name("ListActivities")?, path: path("activityArn")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("activityArn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListStateMachines", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("stateMachines")?),
        queryable: vec![path("name")?, path("type")?, path("creationDate")?],
        fixed: [(path("maxResults")?, FixedValue::Integer(1000))].into_iter().collect(),
        identity: Some(Identity::Arn(path("stateMachineArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListStateMachineAliases", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("stateMachineAliases")?),
        queryable: vec![path("creationDate")?],
        fixed: [(path("maxResults")?, FixedValue::Integer(1000))].into_iter().collect(),
        bind: [(path("stateMachineArn")?, Binding::From { operation: name("ListStateMachines")?, path: path("stateMachineArn")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("stateMachineAliasArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("DescribeStateMachineAlias", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        queryable: vec![path("name")?, path("creationDate")?, path("updateDate")?],
        bind: [(path("stateMachineAliasArn")?, Binding::From { operation: name("ListStateMachineAliases")?, path: path("stateMachineAliasArn")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("stateMachineAliasArn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListStateMachineVersions", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("stateMachineVersions")?),
        queryable: vec![path("creationDate")?],
        fixed: [(path("maxResults")?, FixedValue::Integer(1000))].into_iter().collect(),
        bind: [(path("stateMachineArn")?, Binding::From { operation: name("ListStateMachines")?, path: path("stateMachineArn")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("stateMachineVersionArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("DescribeStateMachine", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Definition is content by default. includedData=METADATA_ONLY documents definition={}, but does not establish suppression of definition-derived variableReferences (state names and variable references). Requires documented server-side suppression or explicit V06 basis before enabling; hash exclusions are not redaction.".to_owned()),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("V31 requires bulk tagging first. Output contains only tags, with no resourceArn for stable parent identity. Requires request-derived identity, union binding from ListStateMachines.stateMachineArn and ListActivities.activityArn, and bulk-coverage fallback gating; tag keys alone are not resource identities.".to_owned()),
        ..empty()
    }),
("ListExecutions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ExecutionListItem is payload-free runtime metadata, not state-machine configuration. V06 architectural-runtime scope needs a reviewed execution-selection policy; unrestricted listing includes completed execution history contrary to V22. STANDARD-only state-machine listing also needs a type predicate; EXPRESS children require mapRunArn instead.".to_owned()),
        ..empty()
    }),
("ListMapRuns", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("MapRunListItem contains payload-free execution/map/state-machine ARNs and dates. Requires a V06 architectural-runtime selection policy and an enabled execution parent: executionArn must come from ListExecutions.executionArn, currently deferred; it cannot bind to a state-machine ARN.".to_owned()),
        ..empty()
    }),
("DescribeMapRun", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Output is payload-free execution progress/configuration, item/execution counts and redrive metadata. Requires the same V06 runtime-scope decision and an enabled ListMapRuns.mapRunArn source; ListMapRuns is deferred, so no complete safe parent plan exists.".to_owned()),
        ..empty()
    }),
("DescribeExecution", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeStateMachineForExecution", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetActivityTask", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("GetExecutionHistory", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("CreateActivity", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateStateMachine", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateStateMachineAlias", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteActivity", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteStateMachine", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
