use super::{NameError, ServiceOverlay};
mod construction;
mod coverage_plans;
mod qapps;
mod qbusiness;
mod qconnect;
mod quicksight;
mod ram;
mod rbin;
mod rds_data;
mod redshift;
mod redshift_data;
mod redshift_serverless;
mod rekognition;
mod repostspace;
mod resiliencehub;
mod resiliencehubv2;
mod resource_explorer_2;
mod resource_groups;
mod resource_groups_tagging_api;
mod rolesanywhere;
mod route53_recovery_cluster;
mod route53_recovery_control_config;
mod route53_recovery_readiness;
mod route53globalresolver;
mod route53profiles;
mod route53resolver;
mod route_53_domains;
mod routing_plans;
mod rtbfabric;
mod rum;
mod s3_control;
mod s3files;
mod s3outposts;
mod s3tables;
mod s3vectors;
mod sagemaker;
mod sagemaker_a2i_runtime;
mod sagemaker_edge;
mod sagemaker_featurestore_runtime;
mod sagemaker_geospatial;
mod sagemaker_metrics;
mod sagemaker_runtime;
mod sagemaker_runtime_http2;
mod sagemakerjobruntime;
mod sagemakertrainingsessionruntime;
mod savingsplans;
mod scheduler;
mod schemas;
mod secrets_manager;
mod security_ir;
mod securityagent;
mod securityhub;
mod securitylake;
mod serverlessapplicationrepository;
mod service_catalog;
mod service_catalog_appregistry;
mod service_quotas;
mod servicediscovery;
mod ses;
mod sesv2;
mod shield;
mod signer;
mod signer_data;
mod signin;
mod simpledbv2;
mod snow_device_management;
mod snowball;
mod socialmessaging;
mod ssm;
mod ssm_contacts;
mod ssm_guiconnect;
mod ssm_incidents;
mod ssm_quicksetup;
mod ssm_sap;
mod sso;
mod sso_admin;
mod sso_oidc;
mod storage_gateway;
mod supplychain;
mod support;
mod support_app;
mod supportauthz;
mod sustainability;
mod swf;
mod synthetics;
mod tag_plans;

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    [
        qapps::overlay(),
        qbusiness::overlay(),
        qconnect::overlay(),
        quicksight::overlay(),
        ram::overlay(),
        rbin::overlay(),
        rds_data::overlay(),
        redshift::overlay(),
        redshift_data::overlay(),
        redshift_serverless::overlay(),
        rekognition::overlay(),
        repostspace::overlay(),
        resiliencehub::overlay(),
        resiliencehubv2::overlay(),
        resource_explorer_2::overlay(),
        resource_groups::overlay(),
        resource_groups_tagging_api::overlay(),
        rolesanywhere::overlay(),
        route_53_domains::overlay(),
        route53_recovery_cluster::overlay(),
        route53_recovery_control_config::overlay(),
        route53_recovery_readiness::overlay(),
        route53globalresolver::overlay(),
        route53profiles::overlay(),
        route53resolver::overlay(),
        rtbfabric::overlay(),
        rum::overlay(),
        s3_control::overlay(),
        s3files::overlay(),
        s3outposts::overlay(),
        s3tables::overlay(),
        s3vectors::overlay(),
        sagemaker::overlay(),
        sagemaker_a2i_runtime::overlay(),
        sagemaker_edge::overlay(),
        sagemaker_featurestore_runtime::overlay(),
        sagemaker_geospatial::overlay(),
        sagemaker_metrics::overlay(),
        sagemaker_runtime::overlay(),
        sagemaker_runtime_http2::overlay(),
        sagemakerjobruntime::overlay(),
        sagemakertrainingsessionruntime::overlay(),
        savingsplans::overlay(),
        scheduler::overlay(),
        schemas::overlay(),
        secrets_manager::overlay(),
        security_ir::overlay(),
        securityagent::overlay(),
        securityhub::overlay(),
        securitylake::overlay(),
        serverlessapplicationrepository::overlay(),
        service_catalog::overlay(),
        service_catalog_appregistry::overlay(),
        service_quotas::overlay(),
        servicediscovery::overlay(),
        ses::overlay(),
        sesv2::overlay(),
        shield::overlay(),
        signer::overlay(),
        signer_data::overlay(),
        signin::overlay(),
        simpledbv2::overlay(),
        snow_device_management::overlay(),
        snowball::overlay(),
        socialmessaging::overlay(),
        ssm::overlay(),
        ssm_contacts::overlay(),
        ssm_guiconnect::overlay(),
        ssm_incidents::overlay(),
        ssm_quicksetup::overlay(),
        ssm_sap::overlay(),
        sso::overlay(),
        sso_admin::overlay(),
        sso_oidc::overlay(),
        storage_gateway::overlay(),
        supplychain::overlay(),
        support::overlay(),
        support_app::overlay(),
        supportauthz::overlay(),
        sustainability::overlay(),
        swf::overlay(),
        synthetics::overlay(),
    ]
    .into_iter()
    .map(|service| tag_plans::apply(coverage_plans::apply(routing_plans::apply(service?)?)?))
    .collect()
}

#[cfg(test)]
mod tests;
