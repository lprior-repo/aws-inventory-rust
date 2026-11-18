use std::collections::BTreeMap;

use aws_sdk_lambda::{
    Client, error::SdkError, operation::list_functions::ListFunctionsError,
    types::FunctionConfiguration,
};
use inventory_core::Source;
use inventory_engine::limiter::ApiError;
use serde_json::json;

use crate::{
    AwsScanError,
    common::{
        Context, PageBody, paginate, parse_retry_after_seconds, require_budget, required,
        sdk_error_with_hint,
    },
};

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_lambda_function", None, Source::Direct),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("lambda", "control_read", || async {
                        client
                            .list_functions()
                            .max_items(ctx.page_size.clamp(1, 50))
                            .set_marker(token.clone())
                            .send()
                            .await
                            .map_err(list_functions_error)
                    })
                    .await?;
                require_budget(output.functions().len(), budget)?;
                let mut body = PageBody {
                    next_token: output.next_marker().map(str::to_owned),
                    listed_count: output.functions().len(),
                    ..PageBody::default()
                };
                for function in output.functions() {
                    map_function(ctx, &mut body, function)?;
                }
                Ok(body)
            }
        },
    )
    .await
}

fn list_functions_error(error: SdkError<ListFunctionsError>) -> ApiError {
    let hint = error.as_service_error().and_then(|service| match service {
        ListFunctionsError::TooManyRequestsException(throttled) => throttled
            .retry_after_seconds()
            .map(parse_retry_after_seconds),
        _ => None,
    });
    sdk_error_with_hint(error, hint)
}

fn map_function(
    ctx: &Context,
    body: &mut PageBody,
    function: &FunctionConfiguration,
) -> Result<(), AwsScanError> {
    let arn = required(function.function_arn(), "Lambda function ARN")?;
    let vpc = function.vpc_config();
    let resource = ctx.resource("aws_lambda_function", arn, function.function_name().map(str::to_owned), json!({
        "function_name": function.function_name(), "runtime": function.runtime().map(|value|value.as_str()),
        "handler": function.handler(), "memory_size": function.memory_size(), "timeout": function.timeout(),
        "role": function.role(), "code_size": function.code_size(), "description": function.description(),
        "version": function.version(), "last_modified": function.last_modified(),
        "state": function.state().map(|value|value.as_str()),
        "package_type": function.package_type().map(|value|value.as_str()),
        "architectures": function.architectures().iter().map(|value|value.as_str()).collect::<Vec<_>>(),
        "vpc_id": vpc.and_then(|value|value.vpc_id()),
        "subnet_ids": vpc.as_ref().map(|value| value.subnet_ids().to_vec()),
        "security_group_ids": vpc.as_ref().map(|value| value.security_group_ids().to_vec()),
        "tags_collected": false,
    }), BTreeMap::new())?;
    if let Some(role) = function.role() {
        body.edges
            .push(ctx.edge(&resource, role.to_owned(), "ASSUMES_ROLE"));
    }
    if let Some(vpc) = vpc {
        if let Some(id) = vpc.vpc_id().filter(|value| !value.is_empty()) {
            body.edges
                .push(ctx.edge(&resource, ctx.ec2_arn("vpc", id), "IN_VPC"));
        }
        for subnet in vpc.subnet_ids() {
            body.edges
                .push(ctx.edge(&resource, ctx.ec2_arn("subnet", subnet), "USES_SUBNET"));
        }
        for group in vpc.security_group_ids() {
            body.edges.push(ctx.edge(
                &resource,
                ctx.ec2_arn("security-group", group),
                "USES_SECURITY_GROUP",
            ));
        }
    }
    body.resources.push(resource);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::list_functions_error;
    use aws_sdk_lambda::{
        config::http::HttpResponse, error::SdkError, operation::list_functions::ListFunctionsError,
        types::error::TooManyRequestsException,
    };
    use aws_smithy_types::{body::SdkBody, error::ErrorMetadata};
    use inventory_engine::limiter::{ApiError, RetryAfter, RetryKind};
    use std::time::Duration;

    #[test]
    fn typed_lambda_retry_delay_survives_error_classification()
    -> Result<(), Box<dyn std::error::Error>> {
        for (seconds, expected) in [
            ("3", RetryAfter::Delay(Duration::from_secs(3))),
            ("301", RetryAfter::Excessive),
            ("invalid", RetryAfter::Invalid),
        ] {
            let modeled = TooManyRequestsException::builder()
                .retry_after_seconds(seconds)
                .meta(
                    ErrorMetadata::builder()
                        .code("TooManyRequestsException")
                        .message("fixture Lambda throttling")
                        .build(),
                )
                .build();
            let response = HttpResponse::new(429_u16.try_into()?, SdkBody::empty());
            let error = SdkError::service_error(
                ListFunctionsError::TooManyRequestsException(modeled),
                response,
            );
            assert!(matches!(list_functions_error(error), ApiError::RetryAfter {
                kind: RetryKind::Throttled, message, retry_after,
            } if retry_after == expected && message.contains("fixture Lambda throttling")));
        }
        Ok(())
    }

    #[test]
    fn lambda_typed_and_header_hints_use_the_longer_delay() -> Result<(), Box<dyn std::error::Error>>
    {
        let modeled = TooManyRequestsException::builder()
            .retry_after_seconds("2")
            .build();
        let mut response = HttpResponse::new(429_u16.try_into()?, SdkBody::empty());
        response
            .headers_mut()
            .try_insert("x-amz-retry-after", "3500")?;
        let error = SdkError::service_error(
            ListFunctionsError::TooManyRequestsException(modeled),
            response,
        );
        assert!(matches!(list_functions_error(error), ApiError::RetryAfter {
            kind: RetryKind::Throttled, retry_after: RetryAfter::Delay(delay), ..
        } if delay == Duration::from_millis(3_500)));
        Ok(())
    }
}
