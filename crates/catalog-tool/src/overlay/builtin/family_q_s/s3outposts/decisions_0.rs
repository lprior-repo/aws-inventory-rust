// Pinned operation purposes: .local/aws-models/models/s3outposts/service/2017-07-25/s3outposts-2017-07-25.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates an endpoint and associates it with the specified Outpost. It can take up to 5 minutes for this action to finish. Related actions include: DeleteEndpoint ListEndpoints
    ("CreateEndpoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an endpoint. It can take up to 5 minutes for this action to finish. Related actions include: CreateEndpoint ListEndpoints
    ("DeleteEndpoint", Decision::Denied(DenyReason::Mutating)),
    ("ListSharedEndpoints", Decision::Deferred("Required selectors OutpostId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Endpoints, NextToken. Pinned purpose: Lists all endpoints associated with an Outpost that has been shared by Amazon Web Services Resource Access Manager (RAM). Related actions include: CreateEndpoint DeleteEndpoint")),
];
