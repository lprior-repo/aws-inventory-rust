// Pinned operation purposes: .local/aws-models/models/sagemaker-edge/service/2020-09-23/sagemaker-edge-2020-09-23.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("GetDeployments", Decision::Deferred("Required selectors DeviceName, DeviceFleetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Deployments. Pinned purpose: Use to get the active deployments from a device.")),
    ("GetDeviceRegistration", Decision::Deferred("Required selectors DeviceName, DeviceFleetName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DeviceRegistration, CacheTTL. Pinned purpose: Use to check if a device is registered with SageMaker Edge Manager.")),
    // Changes service state rather than reading inventory. Pinned purpose: Use to get the current status of devices registered on SageMaker Edge Manager.
    ("SendHeartbeat", Decision::Denied(DenyReason::Mutating)),
];
