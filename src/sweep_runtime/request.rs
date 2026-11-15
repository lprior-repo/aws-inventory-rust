pub(super) mod fixed_input;
#[cfg(test)]
mod fixed_input_tests;
#[cfg(test)]
mod page_size_tests;
use bytes::Bytes;
use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeProtocol},
    record::Record,
};

use super::{
    SweepError,
    input::{InputField, merge_input},
};

pub(super) fn encode(
    plan: &CallablePlan<'_>,
    endpoint: &str,
    base: &Record,
    fields: &[InputField],
) -> Result<http::Request<Bytes>, SweepError> {
    let input = merge_input(base, fields)?;
    let context = aws_wire::request::RequestContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol: protocol(plan.protocol()),
        input: input.root(),
        endpoint,
        limits: aws_wire::request::RequestLimits::default(),
        idempotency_token: None,
        compression: aws_wire::request::CompressionPolicy::default(),
        addressing: None,
    };
    match context.protocol {
        aws_wire::request::Protocol::AwsJson10
        | aws_wire::request::Protocol::AwsJson11
        | aws_wire::request::Protocol::RpcV2Cbor => {
            aws_wire::request::body::encode(context).map_err(SweepError::from)
        }
        aws_wire::request::Protocol::AwsQuery | aws_wire::request::Protocol::Ec2Query => {
            aws_wire::request::query::encode(context).map_err(SweepError::from)
        }
        aws_wire::request::Protocol::RestJson1 | aws_wire::request::Protocol::RestXml => {
            aws_wire::request::rest::encode(context).map_err(SweepError::from)
        }
    }
}

fn protocol(protocol: RuntimeProtocol) -> aws_wire::request::Protocol {
    match protocol {
        RuntimeProtocol::AwsJson10 => aws_wire::request::Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => aws_wire::request::Protocol::AwsJson11,
        RuntimeProtocol::RpcV2Cbor => aws_wire::request::Protocol::RpcV2Cbor,
        RuntimeProtocol::AwsQuery => aws_wire::request::Protocol::AwsQuery,
        RuntimeProtocol::Ec2Query => aws_wire::request::Protocol::Ec2Query,
        RuntimeProtocol::RestJson1 => aws_wire::request::Protocol::RestJson1,
        RuntimeProtocol::RestXml => aws_wire::request::Protocol::RestXml,
    }
}
