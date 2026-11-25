use super::{CallablePlan, DecodedResponse, Response, SweepError};
use aws_wire::response::{ResponseContext, errors};

mod messages;

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
    error: errors::ServiceError,
    context: ResponseContext<'_>,
    account: &str,
    region: &str,
) -> Result<ValidatedResponse, inventory_engine::limiter::ApiError> {
    let service = plan.service_name();
    let operation = plan.operation_name();
    let absent = context.status.is_client_error()
        && context.status != http::StatusCode::TOO_MANY_REQUESTS
        && error.code.as_deref().is_some_and(|code| {
            if known(service, operation, code) {
                !errors::header_body_codes_disagree(context)
            } else {
                messages::confirms(service, operation, code, context, account, region)
            }
        });
    if absent {
        Ok(ValidatedResponse::Absent(error.to_string()))
    } else {
        Err(super::classify_service(error, context.status))
    }
}

fn known(service: &str, operation: &str, code: &str) -> bool {
    match service {
        "iam" => operation == "GetLoginProfile" && code == "NoSuchEntity",
        "organizations" => organization(operation, code),
        "drs" | "mgn" => code == "UninitializedAccountException",
        "compute-optimizer" | "compute-optimizer-automation" => code == "OptInRequiredException",
        "ec2" => {
            operation == "DescribeSpotDatafeedSubscription"
                && code == "InvalidSpotDatafeed.NotFound"
        }
        "resource-explorer-2" => operation == "GetIndex" && code == "ResourceNotFoundException",
        "s3-control" => {
            matches!(
                operation,
                "GetAccessGrantsInstance" | "ListAccessGrants" | "ListAccessGrantsLocations"
            ) && code == "AccessGrantsInstanceNotExistsError"
        }
        _ => false,
    }
}

fn organization(operation: &str, code: &str) -> bool {
    match code {
        "AWSOrganizationsNotInUseException" => true,
        "ResourcePolicyNotFoundException" => operation == "DescribeResourcePolicy",
        "AccountNotRegisteredException" => operation == "ListDelegatedServicesForAccount",
        "EffectivePolicyNotFoundException" => operation == "DescribeEffectivePolicy",
        _ => false,
    }
}

#[cfg(test)]
mod tests;
