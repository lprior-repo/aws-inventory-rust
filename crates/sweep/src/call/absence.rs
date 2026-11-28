use super::{CallablePlan, DecodedResponse, Response, SweepError};
use aws_wire::response::{ResponseContext, errors};

mod messages;

pub(super) enum ValidatedResponse {
    Page(Response),
    Absent(String),
    NotEnabled(String),
}

impl ValidatedResponse {
    pub(super) fn decode(
        self,
        plan: &CallablePlan<'_>,
        emulator_numeric_timestamps: bool,
    ) -> Result<DecodedResponse, SweepError> {
        match self {
            Self::Page(response) => {
                super::decode_response(plan, response, emulator_numeric_timestamps)
            }
            Self::Absent(reason) => Err(SweepError::Absent(reason)),
            Self::NotEnabled(reason) => Err(SweepError::NotEnabled(reason)),
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
    if context.status.is_client_error()
        && context.status != http::StatusCode::TOO_MANY_REQUESTS
        && !errors::header_body_codes_disagree(context)
    {
        if error
            .code
            .as_deref()
            .is_some_and(|code| known(service, operation, code))
        {
            return Ok(ValidatedResponse::Absent(error.to_string()));
        }
        if let Some(code) = error.code.as_deref() {
            match messages::classify(service, operation, code, context, account, region) {
                Some(messages::Classification::Empty) => {
                    return Ok(ValidatedResponse::Absent(error.to_string()));
                }
                Some(messages::Classification::NotEnabled(reason))
                    if !errors::header_codes_conflict(context) =>
                {
                    return Ok(ValidatedResponse::NotEnabled(reason));
                }
                Some(messages::Classification::NotEnabled(_)) | None => {}
            }
        }
    }
    Err(super::classify_service(error, context.status))
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
