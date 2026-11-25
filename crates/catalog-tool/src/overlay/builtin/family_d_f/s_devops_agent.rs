use super::support::{inventory, key, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "devops-agent",
        vec![
            queried(
                inventory(
                    "ListAgentSpaces",
                    "agentSpaces",
                    key(&["agentSpaceId"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["agentSpaceId", "name"],
            )?,
            queried(
                inventory(
                    "ListPrivateConnections",
                    "privateConnections",
                    key(&["name"])?,
                    Paginator::None,
                )?,
                &[
                    "name",
                    "type",
                    "resourceGatewayId",
                    "vpcId",
                    "resourceConfigurationId",
                    "status",
                ],
            )?,
        ],
        &[
            super::s_devops_agent_decisions_0::ENTRIES,
            super::s_devops_agent_decisions_1::ENTRIES,
        ],
        &[("ListServices", DenyReason::Secret)],
    )
}
