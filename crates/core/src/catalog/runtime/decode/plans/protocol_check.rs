use super::super::super::types::RuntimeProtocol;
use super::super::RuntimeCatalogError;
use crate::catalog::Service;

pub(super) fn validate(
    service: Service<'_>,
    protocol: RuntimeProtocol,
) -> Result<(), RuntimeCatalogError> {
    let expected = match protocol {
        RuntimeProtocol::AwsJson10 => "aws.protocols#awsJson1_0",
        RuntimeProtocol::AwsJson11 => "aws.protocols#awsJson1_1",
        RuntimeProtocol::RpcV2Cbor => "smithy.protocols#rpcv2Cbor",
        RuntimeProtocol::AwsQuery => "aws.protocols#awsQuery",
        RuntimeProtocol::Ec2Query => "aws.protocols#ec2Query",
        RuntimeProtocol::RestJson1 => "aws.protocols#restJson1",
        RuntimeProtocol::RestXml => "aws.protocols#restXml",
    };
    let matched = service.protocols()?.try_fold(false, |found, candidate| {
        Ok::<_, RuntimeCatalogError>(found || candidate? == expected)
    })?;
    if matched {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}
