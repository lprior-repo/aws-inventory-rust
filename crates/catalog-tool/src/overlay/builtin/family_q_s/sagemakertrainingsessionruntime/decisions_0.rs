use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ExecuteSamplingOperation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExecuteTrainingOperation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetSamplingOperationResult",
        Decision::Deferred(
            "Required selectors trainingSessionName, operationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members status, operationResult, errorDetails. Pinned purpose: Retrieves the current status and result of a previously submitted sampling operation. Poll this operation until the status reaches a terminal state (Completed or Failed).",
        ),
    ),
    (
        "GetTrainingOperationResult",
        Decision::Deferred(
            "Required selectors trainingSessionName, operationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members status, operationResult, errorDetails, tokenUsage. Pinned purpose: Retrieves the current status and result of a previously submitted training operation. Poll this operation until the status reaches a terminal state (Completed, Failed, or Skipped).",
        ),
    ),
    (
        "SampleTrainingSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SampleTrainingSessionWithResponseStream",
        Decision::Deferred(
            "Required selectors trainingSessionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members contentType, body. Pinned purpose: Streams inference responses from the model associated with the specified training session. Use this operation for real-time token-by-token generation. The service passes the request body to the model without modification.",
        ),
    ),
];
