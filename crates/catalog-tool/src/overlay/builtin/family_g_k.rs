//! Reviewed pinned-model decisions, not emulator-verified service support.
mod gamelift;
mod gameliftstreams;
mod geo_maps;
mod geo_places;
mod geo_routes;
mod glacier;
mod global_accelerator;
mod glue;
mod grafana;
mod greengrass;
mod greengrassv2;
mod groundstation;
mod guardduty;
mod health;
mod healthlake;
mod iam_toolbox;
mod identitystore;
mod imagebuilder;
mod inspector;
mod inspector2;
mod inspector_scan;
mod interconnect;
mod internetmonitor;
mod invoicing;
mod iot;
mod iot_data_plane;
mod iot_jobs_data_plane;
mod iot_managed_integrations;
mod iot_wireless;
mod iotdeviceadvisor;
mod iotfleetwise;
mod iotsecuretunneling;
mod iotsitewise;
mod iotthingsgraph;
mod iottwinmaker;
mod ivs;
mod ivs_realtime;
mod ivschat;
mod kafka;
mod kafkaconnect;
mod kendra;
mod kendra_ranking;
mod keyspaces;
mod keyspacesstreams;
mod kinesis;
mod kinesis_analytics;
mod kinesis_analytics_v2;
mod kinesis_video;
mod kinesis_video_archived_media;
mod kinesis_video_media;
mod kinesis_video_signaling;
mod kinesis_video_webrtc_storage;
mod policy;
#[cfg(test)]
mod tests;

use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        gamelift::overlay()?,
        gameliftstreams::overlay()?,
        geo_maps::overlay()?,
        geo_places::overlay()?,
        geo_routes::overlay()?,
        glacier::overlay()?,
        global_accelerator::overlay()?,
        glue::overlay()?,
        grafana::overlay()?,
        greengrass::overlay()?,
        greengrassv2::overlay()?,
        groundstation::overlay()?,
        guardduty::overlay()?,
        health::overlay()?,
        healthlake::overlay()?,
        iam_toolbox::overlay()?,
        identitystore::overlay()?,
        imagebuilder::overlay()?,
        inspector::overlay()?,
        inspector_scan::overlay()?,
        inspector2::overlay()?,
        interconnect::overlay()?,
        internetmonitor::overlay()?,
        invoicing::overlay()?,
        iot::overlay()?,
        iot_data_plane::overlay()?,
        iot_jobs_data_plane::overlay()?,
        iot_managed_integrations::overlay()?,
        iot_wireless::overlay()?,
        iotdeviceadvisor::overlay()?,
        iotfleetwise::overlay()?,
        iotsecuretunneling::overlay()?,
        iotsitewise::overlay()?,
        iotthingsgraph::overlay()?,
        iottwinmaker::overlay()?,
        ivs::overlay()?,
        ivs_realtime::overlay()?,
        ivschat::overlay()?,
        kafka::overlay()?,
        kafkaconnect::overlay()?,
        kendra::overlay()?,
        kendra_ranking::overlay()?,
        keyspaces::overlay()?,
        keyspacesstreams::overlay()?,
        kinesis::overlay()?,
        kinesis_analytics::overlay()?,
        kinesis_analytics_v2::overlay()?,
        kinesis_video::overlay()?,
        kinesis_video_archived_media::overlay()?,
        kinesis_video_media::overlay()?,
        kinesis_video_signaling::overlay()?,
        kinesis_video_webrtc_storage::overlay()?,
    ])
}
