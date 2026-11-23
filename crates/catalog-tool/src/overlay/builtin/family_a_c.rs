mod accessanalyzer_0;
mod account_0;
mod account_access_0;
mod account_access_runtime_0;
mod acm_0;
mod acm_pca_0;
mod agent_registry_0;
mod agent_registry_control_0;
mod aiops_0;
mod amp_0;
mod amplify_0;
mod amplifybackend_0;
mod amplifyuibuilder_0;
mod api_gateway_0;
mod apigatewaymanagementapi_0;
mod apigatewayv2_0;
mod app_mesh_0;
mod appconfig_0;
mod appconfigdata_0;
mod appfabric_0;
mod appflow_0;
mod appintegrations_0;
mod application_auto_scaling_0;
mod application_discovery_service_0;
mod application_insights_0;
mod application_signals_0;
mod applicationcostprofiler_0;
mod apprunner_0;
mod appstream_0;
mod appsync_0;
mod arc_region_switch_0;
mod arc_zonal_shift_0;
mod artifact_0;
mod athena_0;
mod auditmanager_0;
mod auto_scaling_0;
mod auto_scaling_plans_0;
mod b2bi_0;
mod backup_0;
mod backup_gateway_0;
mod backupsearch_0;
mod batch_0;
mod batch_1;
mod bcm_dashboards_0;
mod bcm_data_exports_0;
mod bcm_pricing_calculator_0;
mod bcm_recommended_actions_0;
mod bedrock_0;
mod bedrock_agent_0;
mod bedrock_agent_runtime_0;
mod bedrock_agentcore_0;
mod bedrock_agentcore_control_0;
mod bedrock_agentcore_control_1;
mod bedrock_data_automation_0;
mod bedrock_data_automation_runtime_0;
mod bedrock_runtime_0;
mod billing_0;
mod billingconductor_0;
mod braket_0;
mod budgets_0;
mod chatbot_0;
mod chime_0;
mod chime_sdk_identity_0;
mod chime_sdk_media_pipelines_0;
mod chime_sdk_meetings_0;
mod chime_sdk_messaging_0;
mod chime_sdk_voice_0;
mod cleanrooms_0;
mod cleanroomsml_0;
mod cloud9_0;
mod cloudcontrol_0;
mod clouddirectory_0;
mod cloudformation_0;
mod cloudformation_1;
mod cloudformation_2;
mod cloudfront_keyvaluestore_0;
mod cloudhsm_0;
mod cloudhsm_v2_0;
mod cloudsearch_0;
mod cloudsearch_domain_0;
mod cloudtrail_0;
mod cloudtrail_data_0;
mod cloudwatch_events_0;
mod cloudwatch_logs_0;
mod cloudwatchomni_0;
mod codeartifact_0;
mod codebuild_0;
mod codecatalyst_0;
mod codecommit_0;
mod codeconnections_0;
mod codedeploy_0;
mod codeguru_reviewer_0;
mod codeguru_security_0;
mod codeguruprofiler_0;
mod codepipeline_0;
mod codestar_connections_0;
mod codestar_notifications_0;
mod cognito_identity_0;
mod cognito_identity_provider_0;
mod cognito_sync_0;
mod comprehend_0;
mod comprehendmedical_0;
mod compute_optimizer_0;
mod compute_optimizer_automation_0;
mod config_service_0;
mod connect_0;
mod connect_1;
mod connect_2;
mod connect_contact_lens_0;
mod connectcampaigns_0;
mod connectcampaignsv2_0;
mod connectcases_0;
mod connecthealth_0;
mod connectparticipant_0;
mod continuum_0;
mod controlcatalog_0;
mod controltower_0;
mod cost_and_usage_report_service_0;
mod cost_explorer_0;
mod cost_optimization_hub_0;
mod customer_profiles_0;
mod registry_0;
mod registry_1;
mod registry_10;
mod registry_11;
mod registry_2;
mod registry_3;
mod registry_4;
mod registry_5;
mod registry_6;
mod registry_7;
mod registry_8;
mod registry_9;
mod support;

#[cfg(test)]
mod consumer_fixture;
#[cfg(test)]
mod consumer_tests;
#[cfg(test)]
use crate as fixture_catalog;

use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok([
        registry_0::overlays()?,
        registry_1::overlays()?,
        registry_2::overlays()?,
        registry_3::overlays()?,
        registry_4::overlays()?,
        registry_5::overlays()?,
        registry_6::overlays()?,
        registry_7::overlays()?,
        registry_8::overlays()?,
        registry_9::overlays()?,
        registry_10::overlays()?,
        registry_11::overlays()?,
    ]
    .into_iter()
    .flatten()
    .collect())
}
