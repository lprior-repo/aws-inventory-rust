//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/groundstation/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListConfigs", policy::query(policy::enumerate("configList", policy::arn("configArn")?, Some(("nextToken", "nextToken")))?, &["configArn"])?),
        ("ListDataflowEndpointGroups", policy::query(policy::enumerate("dataflowEndpointGroupList", policy::arn("dataflowEndpointGroupArn")?, Some(("nextToken", "nextToken")))?, &["dataflowEndpointGroupArn"])?),
        ("ListMissionProfiles", policy::query(policy::enumerate("missionProfileList", policy::arn("missionProfileArn")?, Some(("nextToken", "nextToken")))?, &["missionProfileArn"])?),
        ("ListSatellites", policy::query(policy::enumerate("satellites", policy::arn("satelliteArn")?, Some(("nextToken", "nextToken")))?, &["satelliteArn"])?),
        ("GetMissionProfile", policy::query(policy::tags(policy::bind(policy::enumerate("$", policy::arn("missionProfileArn")?, None)?, "missionProfileId", "ListMissionProfiles", "missionProfileId")?, "tags")?, &["missionProfileArn"])?),
    ];
    policy::finish(
        "groundstation", approved, DEFERRED,
        "Configuration, endpoint-group, mission-profile and satellite summaries are metadata. Contacts/ephemerides require explicit bounded time windows; agent upload capabilities are denied; remaining children are unapproved.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Secret, SECRET)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeContact", "DescribeContactVersion", "DescribeEphemeris", "GetAgentConfiguration", "GetConfig",
    "GetDataflowEndpointGroup", "GetMinuteUsage", "GetSatellite", "ListAntennas", "ListContactVersions",
    "ListContacts", "ListEphemerides", "ListGroundStationReservations", "ListGroundStations",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CancelContact", "CreateConfig", "CreateDataflowEndpointGroup", "CreateDataflowEndpointGroupV2",
    "CreateEphemeris", "CreateMissionProfile", "DeleteConfig", "DeleteDataflowEndpointGroup",
    "DeleteEphemeris", "DeleteMissionProfile", "RegisterAgent", "ReserveContact", "TagResource",
    "UntagResource", "UpdateAgentStatus", "UpdateConfig", "UpdateContact", "UpdateEphemeris",
    "UpdateMissionProfile",
];

const SECRET: &[&str] = &[
    "GetAgentTaskResponseUrl",
];
