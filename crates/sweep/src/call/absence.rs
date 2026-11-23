use super::{CallablePlan, DecodedResponse, Response, SweepError};

pub(super) enum ValidatedResponse {
    Page(Response),
    Absent(String),
}

impl ValidatedResponse {
    pub(super) fn decode(self, plan: &CallablePlan<'_>) -> Result<DecodedResponse, SweepError> {
        match self {
            Self::Page(response) => super::decode_response(plan, response),
            Self::Absent(reason) => Err(SweepError::Absent(reason)),
        }
    }
}

pub(super) fn classify(
    plan: &CallablePlan<'_>,
    error: aws_wire::response::errors::ServiceError,
) -> Result<ValidatedResponse, inventory_engine::limiter::ApiError> {
    if known(
        plan.service_name(),
        plan.operation_name(),
        error.code.as_deref(),
    ) {
        Ok(ValidatedResponse::Absent(error.to_string()))
    } else {
        Err(super::classify_service(error))
    }
}

// These codes identify an absent feature or singleton, not a missing caller-supplied ID.
// Permission errors and generic ResourceNotFound errors must remain failures.
pub(super) fn known(service: &str, operation: &str, code: Option<&str>) -> bool {
    match service {
        "iam" => operation == "GetLoginProfile" && code == Some("NoSuchEntity"),
        "organizations" => organization(operation, code),
        "drs" | "mgn" => code == Some("UninitializedAccountException"),
        "compute-optimizer" | "compute-optimizer-automation" => {
            code == Some("OptInRequiredException")
        }
        "ec2" => {
            operation == "DescribeSpotDatafeedSubscription"
                && code == Some("InvalidSpotDatafeed.NotFound")
        }
        "resource-explorer-2" => {
            operation == "GetIndex" && code == Some("ResourceNotFoundException")
        }
        _ => false,
    }
}

fn organization(operation: &str, code: Option<&str>) -> bool {
    match code {
        Some("AWSOrganizationsNotInUseException") => true,
        Some("ResourcePolicyNotFoundException") => operation == "DescribeResourcePolicy",
        Some("AccountNotRegisteredException") => operation == "ListDelegatedServicesForAccount",
        Some("EffectivePolicyNotFoundException") => operation == "DescribeEffectivePolicy",
        _ => false,
    }
}

#[cfg(test)]
mod tests;
