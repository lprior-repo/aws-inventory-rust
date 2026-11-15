// Pinned model: .local/aws-models/models/devops-agent/service/2026-01-01/devops-agent-2026-01-01.json
use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, Paginator, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "devops-agent",
        vec![
        // Lists all AgentSpaces with optional pagination.
        queried(inventory("ListAgentSpaces", "agentSpaces", key(&["agentSpaceId"])?, token("nextToken", "nextToken")?)?, &["agentSpaceId", "name"])?,
        // Lists all Private Connections in the caller's account.
        queried(inventory("ListPrivateConnections", "privateConnections", key(&["name"])?, Paginator::None)?, &["name", "type", "resourceGatewayId", "vpcId", "resourceConfigurationId", "status"])?,
        ],
        &[super::s_devops_agent_decisions_0::ENTRIES, super::s_devops_agent_decisions_1::ENTRIES],
        &[
            // Service details include mcpserversigv4.customHeaders values with no pinned guarantee of secret omission.
            ("ListServices", DenyReason::Secret),
        ],
    )
}
