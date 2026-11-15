// Embedded, explicitly reviewed d <= service key < g policies.
// First-wave dynamodb/ec2/ecs/eks/elastic-load-balancing-v2 remain owned by builtin.rs.
mod support;
mod s_data_pipeline;
mod s_data_pipeline_decisions_0;
mod s_database_migration_service;
mod s_database_migration_service_decisions_0;
mod s_database_migration_service_decisions_1;
mod s_database_migration_service_decisions_2;
mod s_database_migration_service_decisions_3;
mod s_databrew;
mod s_databrew_decisions_0;
mod s_databrew_decisions_1;
mod s_dataexchange;
mod s_dataexchange_decisions_0;
mod s_datasync;
mod s_datasync_decisions_0;
mod s_datasync_decisions_1;
mod s_datazone;
mod s_datazone_decisions_0;
mod s_datazone_decisions_1;
mod s_datazone_decisions_2;
mod s_datazone_decisions_3;
mod s_datazone_decisions_4;
mod s_datazone_decisions_5;
mod s_dax;
mod s_dax_decisions_0;
mod s_deadline;
mod s_deadline_decisions_0;
mod s_deadline_decisions_1;
mod s_deadline_decisions_2;
mod s_deadline_decisions_3;
mod s_detective;
mod s_detective_decisions_0;
mod s_device_farm;
mod s_device_farm_decisions_0;
mod s_device_farm_decisions_1;
mod s_devops_agent;
mod s_devops_agent_decisions_0;
mod s_devops_agent_decisions_1;
mod s_devops_guru;
mod s_devops_guru_decisions_0;
mod s_direct_connect;
mod s_direct_connect_decisions_0;
mod s_direct_connect_decisions_1;
mod s_directory_service;
mod s_directory_service_data;
mod s_directory_service_data_decisions_0;
mod s_directory_service_decisions_0;
mod s_directory_service_decisions_1;
mod s_directory_service_decisions_2;
mod s_dlm;
mod s_dlm_decisions_0;
mod s_docdb;
mod s_docdb_decisions_0;
mod s_docdb_decisions_1;
mod s_docdb_elastic;
mod s_docdb_elastic_decisions_0;
mod s_drs;
mod s_drs_decisions_0;
mod s_drs_decisions_1;
mod s_dsql;
mod s_dsql_decisions_0;
mod s_dynamodb_streams;
mod s_dynamodb_streams_decisions_0;
mod s_ebs;
mod s_ec2_instance_connect;
mod s_ecr;
mod s_ecr_decisions_0;
mod s_ecr_decisions_1;
mod s_ecr_public;
mod s_ecr_public_decisions_0;
mod s_efs;
mod s_efs_decisions_0;
mod s_eks_auth;
mod s_elastic_beanstalk;
mod s_elastic_beanstalk_decisions_0;
mod s_elastic_beanstalk_decisions_1;
mod s_elastic_load_balancing;
mod s_elastic_load_balancing_decisions_0;
mod s_elasticache;
mod s_elasticache_decisions_0;
mod s_elasticache_decisions_1;
mod s_elasticsearch_service;
mod s_elasticsearch_service_decisions_0;
mod s_elasticsearch_service_decisions_1;
mod s_elementalinference;
mod s_elementalinference_decisions_0;
mod s_emr;
mod s_emr_containers;
mod s_emr_containers_decisions_0;
mod s_emr_decisions_0;
mod s_emr_decisions_1;
mod s_emr_serverless;
mod s_emr_serverless_decisions_0;
mod s_endusermessaging;
mod s_endusermessaging_decisions_0;
mod s_entityresolution;
mod s_entityresolution_decisions_0;
mod s_eventbridge;
mod s_eventbridge_decisions_0;
mod s_eventbridge_decisions_1;
mod s_eventbridgev2;
mod s_eventbridgev2_decisions_0;
mod s_evs;
mod s_evs_decisions_0;
mod s_firehose;
mod s_firehose_decisions_0;
mod s_fis;
mod s_fis_decisions_0;
mod s_fms;
mod s_fms_decisions_0;
mod s_fms_decisions_1;
mod s_forecast;
mod s_forecast_decisions_0;
mod s_forecast_decisions_1;
mod s_forecastquery;
mod s_frauddetector;
mod s_frauddetector_decisions_0;
mod s_frauddetector_decisions_1;
mod s_freetier;
mod s_freetier_decisions_0;
mod s_fsx;
mod s_fsx_decisions_0;
mod s_fsx_decisions_1;

use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        s_data_pipeline::overlay()?,
        s_database_migration_service::overlay()?,
        s_databrew::overlay()?,
        s_dataexchange::overlay()?,
        s_datasync::overlay()?,
        s_datazone::overlay()?,
        s_dax::overlay()?,
        s_deadline::overlay()?,
        s_detective::overlay()?,
        s_device_farm::overlay()?,
        s_devops_agent::overlay()?,
        s_devops_guru::overlay()?,
        s_direct_connect::overlay()?,
        s_directory_service::overlay()?,
        s_directory_service_data::overlay()?,
        s_dlm::overlay()?,
        s_docdb::overlay()?,
        s_docdb_elastic::overlay()?,
        s_drs::overlay()?,
        s_dsql::overlay()?,
        s_dynamodb_streams::overlay()?,
        s_ebs::overlay()?,
        s_ec2_instance_connect::overlay()?,
        s_ecr::overlay()?,
        s_ecr_public::overlay()?,
        s_efs::overlay()?,
        s_eks_auth::overlay()?,
        s_elastic_beanstalk::overlay()?,
        s_elastic_load_balancing::overlay()?,
        s_elasticache::overlay()?,
        s_elasticsearch_service::overlay()?,
        s_elementalinference::overlay()?,
        s_emr::overlay()?,
        s_emr_containers::overlay()?,
        s_emr_serverless::overlay()?,
        s_endusermessaging::overlay()?,
        s_entityresolution::overlay()?,
        s_eventbridge::overlay()?,
        s_eventbridgev2::overlay()?,
        s_evs::overlay()?,
        s_firehose::overlay()?,
        s_fis::overlay()?,
        s_fms::overlay()?,
        s_forecast::overlay()?,
        s_forecastquery::overlay()?,
        s_frauddetector::overlay()?,
        s_freetier::overlay()?,
        s_fsx::overlay()?,
    ])
}

#[cfg(test)]
mod tests;
