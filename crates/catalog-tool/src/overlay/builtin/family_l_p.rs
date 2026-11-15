mod policy;
mod lakeformation;
mod lambda_core;
mod lambda_microvms;
mod lambda_web;
mod launch_wizard;
mod lex_model_building_service;
mod lex_models_v2;
mod lex_runtime_service;
mod lex_runtime_v2;
mod license_manager_linux_subscriptions;
mod license_manager_user_subscriptions;
mod license_manager;
mod lightsail;
mod location;
mod lookoutequipment;
mod m2;
mod machine_learning;
mod macie2;
mod mailmanager;
mod managedblockchain_query;
mod managedblockchain;
mod marketplace_agreement;
mod marketplace_catalog;
mod marketplace_commerce_analytics;
mod marketplace_deployment;
mod marketplace_discovery;
mod marketplace_entitlement_service;
mod marketplace_metering;
mod marketplace_reporting;
mod mediaconnect;
mod mediaconvert;
mod medialive;
mod mediapackage_vod;
mod mediapackage;
mod mediapackagev2;
mod mediastore_data;
mod mediastore;
mod mediatailor;
mod medical_imaging;
mod memorydb;
mod mgn;
mod migration_hub_refactor_spaces;
mod migration_hub;
mod migrationhub_config;
mod migrationhuborchestrator;
mod migrationhubstrategy;
mod mpa;
mod mq;
mod mturk;
mod mwaa_serverless;
mod mwaa;
mod neptune_graph;
mod neptune;
mod neptunedata;
mod network_firewall;
mod network_security_manager;
mod networkflowmonitor;
mod networkmanager;
mod networkmonitor;
mod notifications;
mod notificationscontacts;
mod nova_act;
mod oam;
mod observabilityadmin;
mod odb;
mod omics;
mod opensearch;
mod opensearchserverless;
mod osis;
mod outposts;
mod partnercentral_account;
mod partnercentral_benefits;
mod partnercentral_channel;
mod partnercentral_revenue_measurement;
mod partnercentral_selling;
mod payment_cryptography_data;
mod payment_cryptography;
mod pca_connector_ad;
mod pca_connector_scep;
mod pcs;
mod personalize_events;
mod personalize_runtime;
mod personalize;
mod pi;
mod pinpoint_email;
mod pinpoint_sms_voice_v2;
mod pinpoint_sms_voice;
mod pinpoint;
mod pipes;
mod polly;
mod pricing_plan_manager;
mod pricing;
mod proton;

use crate::overlay::{NameError, ServiceOverlay};

type Build = fn() -> Result<ServiceOverlay, NameError>;

// Static reviewed factories: no model-name or operation-prefix authorization at runtime.
const SERVICES: &[Build] = &[
    lakeformation::overlay,
    lambda_core::overlay,
    lambda_microvms::overlay,
    lambda_web::overlay,
    launch_wizard::overlay,
    lex_model_building_service::overlay,
    lex_models_v2::overlay,
    lex_runtime_service::overlay,
    lex_runtime_v2::overlay,
    license_manager_linux_subscriptions::overlay,
    license_manager_user_subscriptions::overlay,
    license_manager::overlay,
    lightsail::overlay,
    location::overlay,
    lookoutequipment::overlay,
    m2::overlay,
    machine_learning::overlay,
    macie2::overlay,
    mailmanager::overlay,
    managedblockchain_query::overlay,
    managedblockchain::overlay,
    marketplace_agreement::overlay,
    marketplace_catalog::overlay,
    marketplace_commerce_analytics::overlay,
    marketplace_deployment::overlay,
    marketplace_discovery::overlay,
    marketplace_entitlement_service::overlay,
    marketplace_metering::overlay,
    marketplace_reporting::overlay,
    mediaconnect::overlay,
    mediaconvert::overlay,
    medialive::overlay,
    mediapackage_vod::overlay,
    mediapackage::overlay,
    mediapackagev2::overlay,
    mediastore_data::overlay,
    mediastore::overlay,
    mediatailor::overlay,
    medical_imaging::overlay,
    memorydb::overlay,
    mgn::overlay,
    migration_hub_refactor_spaces::overlay,
    migration_hub::overlay,
    migrationhub_config::overlay,
    migrationhuborchestrator::overlay,
    migrationhubstrategy::overlay,
    mpa::overlay,
    mq::overlay,
    mturk::overlay,
    mwaa_serverless::overlay,
    mwaa::overlay,
    neptune_graph::overlay,
    neptune::overlay,
    neptunedata::overlay,
    network_firewall::overlay,
    network_security_manager::overlay,
    networkflowmonitor::overlay,
    networkmanager::overlay,
    networkmonitor::overlay,
    notifications::overlay,
    notificationscontacts::overlay,
    nova_act::overlay,
    oam::overlay,
    observabilityadmin::overlay,
    odb::overlay,
    omics::overlay,
    opensearch::overlay,
    opensearchserverless::overlay,
    osis::overlay,
    outposts::overlay,
    partnercentral_account::overlay,
    partnercentral_benefits::overlay,
    partnercentral_channel::overlay,
    partnercentral_revenue_measurement::overlay,
    partnercentral_selling::overlay,
    payment_cryptography_data::overlay,
    payment_cryptography::overlay,
    pca_connector_ad::overlay,
    pca_connector_scep::overlay,
    pcs::overlay,
    personalize_events::overlay,
    personalize_runtime::overlay,
    personalize::overlay,
    pi::overlay,
    pinpoint_email::overlay,
    pinpoint_sms_voice_v2::overlay,
    pinpoint_sms_voice::overlay,
    pinpoint::overlay,
    pipes::overlay,
    polly::overlay,
    pricing_plan_manager::overlay,
    pricing::overlay,
    proton::overlay,
];

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    SERVICES.iter().map(|build| build()).collect()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod consumer_tests;
