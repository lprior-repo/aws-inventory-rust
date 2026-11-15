use super::support::{NameError, ServiceOverlay, service};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        service("accessanalyzer", [
            super::accessanalyzer_0::part_0()?,
            super::accessanalyzer_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("account", [
            super::account_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("account-access", [
            super::account_access_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("account-access-runtime", [
            super::account_access_runtime_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("acm", [
            super::acm_0::part_0()?,
            super::acm_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("acm-pca", [
            super::acm_pca_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("agent-registry", [
            super::agent_registry_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("agent-registry-control", [
            super::agent_registry_control_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("aiops", [
            super::aiops_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("amp", [
            super::amp_0::part_0()?,
            super::amp_0::part_1()?,
        ].into_iter().flatten().collect())?,
    ])
}
