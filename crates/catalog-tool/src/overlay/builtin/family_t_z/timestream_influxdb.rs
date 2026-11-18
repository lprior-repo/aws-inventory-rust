use super::super::service;
use super::{
    defer, deny, timestream_influxdb_plans_1, timestream_influxdb_plans_2,
    timestream_influxdb_plans_3,
};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "timestream-influxdb",
        vec![
            ("CreateDbBackup", deny(DenyReason::Mutating)),
            ("CreateDbCluster", deny(DenyReason::Mutating)),
            ("CreateDbInstance", deny(DenyReason::Mutating)),
            ("CreateDbParameterGroup", deny(DenyReason::Mutating)),
            ("DeleteDbBackup", deny(DenyReason::Mutating)),
            ("DeleteDbCluster", deny(DenyReason::Mutating)),
            ("DeleteDbInstance", deny(DenyReason::Mutating)),
            ("GetDbBackup", timestream_influxdb_plans_1::get_db_backup()?),
            (
                "GetDbCluster",
                timestream_influxdb_plans_1::get_db_cluster()?,
            ),
            (
                "GetDbInstance",
                timestream_influxdb_plans_1::get_db_instance()?,
            ),
            (
                "GetDbParameterGroup",
                timestream_influxdb_plans_2::get_db_parameter_group()?,
            ),
            (
                "ListDbBackups",
                timestream_influxdb_plans_2::list_db_backups()?,
            ),
            (
                "ListDbClusters",
                timestream_influxdb_plans_2::list_db_clusters()?,
            ),
            (
                "ListDbInstances",
                timestream_influxdb_plans_3::list_db_instances()?,
            ),
            (
                "ListDbInstancesForCluster",
                timestream_influxdb_plans_3::list_db_instances_for_cluster()?,
            ),
            (
                "ListDbParameterGroups",
                timestream_influxdb_plans_3::list_db_parameter_groups()?,
            ),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            ("RebootDbCluster", deny(DenyReason::Mutating)),
            ("RebootDbInstance", deny(DenyReason::Mutating)),
            ("RestoreFromDbBackup", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateDbCluster", deny(DenyReason::Mutating)),
            ("UpdateDbInstance", deny(DenyReason::Mutating)),
        ],
    )
}
