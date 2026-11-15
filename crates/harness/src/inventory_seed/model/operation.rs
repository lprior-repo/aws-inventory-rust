use super::*;
pub(in crate::inventory_seed) fn locate<'a>(
    catalog: Catalog<'a>,
    service_name: &str,
    operation_name: &str,
) -> Result<SeedOperation<'a>, SeedError> {
    let service = catalog.service(service_name)?.ok_or(SeedError::Invalid(
        "modeled service is missing from catalog",
    ))?;
    let operation = find_operation(service, operation_name)?;
    let protocol = protocol(service)?;
    Ok(SeedOperation {
        service,
        operation,
        protocol,
    })
}

fn find_operation<'a>(service: Service<'a>, name: &str) -> Result<Operation<'a>, SeedError> {
    service
        .operations()?
        .try_fold(None, |found, candidate| {
            let candidate = candidate?;
            if candidate.name()? == name {
                Ok::<Option<Operation<'a>>, inventory_core::catalog::CatalogError>(Some(candidate))
            } else {
                Ok(found)
            }
        })?
        .ok_or(SeedError::Invalid(
            "modeled operation is missing from catalog",
        ))
}

fn protocol(service: Service<'_>) -> Result<Protocol, SeedError> {
    service
        .protocols()?
        .try_fold(None, |found, candidate| {
            let protocol = match candidate? {
                "aws.protocols#awsQuery" => Some(Protocol::AwsQuery),
                "aws.protocols#ec2Query" => Some(Protocol::Ec2Query),
                "aws.protocols#awsJson1_0" => Some(Protocol::AwsJson10),
                "aws.protocols#awsJson1_1" => Some(Protocol::AwsJson11),
                "smithy.protocols#rpcv2Cbor" => Some(Protocol::RpcV2Cbor),
                "aws.protocols#restJson1" => Some(Protocol::RestJson1),
                "aws.protocols#restXml" => Some(Protocol::RestXml),
                _ => None,
            };
            Ok::<_, SeedError>(found.or(protocol))
        })?
        .ok_or(SeedError::Invalid(
            "seed operation service has no supported protocol",
        ))
}
