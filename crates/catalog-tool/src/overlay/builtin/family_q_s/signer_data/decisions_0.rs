// Pinned operation purposes: .local/aws-models/models/signer-data/service/2017-08-25/signer-data-2017-08-25.json
use super::super::construction::Decision;

pub(super) const DECISIONS: &[(&str, Decision)] = &[(
    "GetRevocationStatus",
    Decision::Deferred(
        "Required selectors signatureTimestamp, platformId, profileVersionArn, jobArn, certificateHashes are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members revokedEntities. Pinned purpose: Retrieves the revocation status for a signed artifact by checking if the signing profile, job, or certificate has been revoked.",
    ),
)];
