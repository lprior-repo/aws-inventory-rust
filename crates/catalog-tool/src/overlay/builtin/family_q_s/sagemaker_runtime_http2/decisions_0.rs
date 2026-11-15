// Pinned operation purposes: .local/aws-models/models/sagemaker-runtime-http2/service/2025-10-01/sagemaker-runtime-http2-2025-10-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Invokes a model endpoint with bidirectional streaming capabilities. This operation establishes a persistent connection that allows you to send multiple requests and receive streaming responses from the model in real-time. Bidirectional streaming is useful for interactive applications such as chatbots, real-time translation, or any scenario where yo
    ("InvokeEndpointWithBidirectionalStream", Decision::Denied(DenyReason::Paid)),
];
