use super::{ContextError, DiscoveryFrame, projection};
use crate::{catalog::runtime::ContextProjectionPlan, inventory::{OperationId, ServiceId}, record::Record};

impl DiscoveryFrame {
    /// Project only operation-wide namespaces for an independently selected resource.
    /// No primary selected row or discovery ordinal is invented for this frame.
    /// The frame is transient identity input; additional emissions never spawn children.
    pub fn project_response(service: ServiceId, operation: OperationId,
        plan: &ContextProjectionPlan, response: &Record, request: &Record,
    ) -> Result<Self, ContextError> {
        Ok(Self { service, operation, ordinal: 0, branch: None,
            selected: projection::project(response, &[])?,
            response: projection::project(response, &plan.response)?,
            request: projection::project(request, &plan.request)?,
        })
    }
}
