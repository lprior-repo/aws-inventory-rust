use super::support::{NameError, ServiceOverlay, service};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        service("codepipeline", [
            super::codepipeline_0::part_0()?,
            super::codepipeline_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("codestar-connections", [
            super::codestar_connections_0::part_0()?,
            super::codestar_connections_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("codestar-notifications", [
            super::codestar_notifications_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("cognito-identity", [
            super::cognito_identity_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("cognito-identity-provider", [
            super::cognito_identity_provider_0::part_0()?,
            super::cognito_identity_provider_0::part_1()?,
            super::cognito_identity_provider_0::part_2()?,
            super::cognito_identity_provider_0::part_3()?,
        ].into_iter().flatten().collect())?,
        service("cognito-sync", [
            super::cognito_sync_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("comprehend", [
            super::comprehend_0::part_0()?,
            super::comprehend_0::part_1()?,
            super::comprehend_0::part_2()?,
            super::comprehend_0::part_3()?,
        ].into_iter().flatten().collect())?,
        service("comprehendmedical", [
            super::comprehendmedical_0::part_0()?,
            super::comprehendmedical_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("compute-optimizer", [
            super::compute_optimizer_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("compute-optimizer-automation", [
            super::compute_optimizer_automation_0::part_0()?,
        ].into_iter().flatten().collect())?,
    ])
}
