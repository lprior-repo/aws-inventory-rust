// Pinned model: .local/aws-models/models/eks-auth/service/2023-11-26/eks-auth-2023-11-26.json
use super::support::{reviewed};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "eks-auth",
        vec![

        ],
        &[],
        &[
            // The Amazon EKS Auth API and the AssumeRoleForPodIdentity action are only used by the EKS Pod Identity Agent.
            ("AssumeRoleForPodIdentity", DenyReason::Secret),
        ],
    )
}
