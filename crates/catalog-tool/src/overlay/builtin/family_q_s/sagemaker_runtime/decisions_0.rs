// Pinned operation purposes: .local/aws-models/models/sagemaker-runtime/service/2017-05-13/sagemaker-runtime-2017-05-13.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: After you deploy a model into production using Amazon SageMaker AI hosting services, your client applications use this API to get inferences from the model hosted at the specified endpoint. For an overview of Amazon SageMaker AI, see How It Works . Amazon SageMaker AI strips all POST headers except those supported by the API. Amazon SageMaker AI mi
    ("InvokeEndpoint", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: After you deploy a model into production using Amazon SageMaker AI hosting services, your client applications use this API to get inferences from the model hosted at the specified endpoint in an asynchronous manner. Inference requests sent to this API are enqueued for asynchronous processing. The processing of the inference request may or may not c
    ("InvokeEndpointAsync", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Invokes a model at the specified endpoint to return the inference response as a stream. The inference stream provides the response payload incrementally as a series of parts. Before you can get an inference stream, you must have access to a model that's deployed using Amazon SageMaker AI hosting services, and the container for that model must suppo
    (
        "InvokeEndpointWithResponseStream",
        Decision::Denied(DenyReason::Paid),
    ),
];
