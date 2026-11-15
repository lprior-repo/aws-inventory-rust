use super::support::{NameError, ServiceOverlay, service};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        service("config-service", [
            super::config_service_0::part_0()?,
            super::config_service_0::part_1()?,
            super::config_service_0::part_2()?,
            super::config_service_0::part_3()?,
        ].into_iter().flatten().collect())?,
        service("connect", [
            super::connect_0::part_0()?,
            super::connect_0::part_1()?,
            super::connect_0::part_2()?,
            super::connect_0::part_3()?,
            super::connect_0::part_4()?,
            super::connect_1::part_0()?,
            super::connect_1::part_1()?,
            super::connect_1::part_2()?,
            super::connect_1::part_3()?,
            super::connect_1::part_4()?,
            super::connect_2::part_0()?,
            super::connect_2::part_1()?,
        ].into_iter().flatten().collect())?,
        service("connect-contact-lens", [
            super::connect_contact_lens_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("connectcampaigns", [
            super::connectcampaigns_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("connectcampaignsv2", [
            super::connectcampaignsv2_0::part_0()?,
            super::connectcampaignsv2_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("connectcases", [
            super::connectcases_0::part_0()?,
            super::connectcases_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("connecthealth", [
            super::connecthealth_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("connectparticipant", [
            super::connectparticipant_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("continuum", [
            super::continuum_0::part_0()?,
            super::continuum_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("controlcatalog", [
            super::controlcatalog_0::part_0()?,
        ].into_iter().flatten().collect())?,
    ])
}
