use super::{TestResult, compile_model, fixture_service};
use inventory_core::catalog::{HttpOperation, Target};

const FIELDS: &str = "Entries\nId\nNext\nPageSize\nToken\nValue\n";

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"example#Service":{"type":"service","version":"2025-04-03","traits":{"aws.api#service":{"sdkId":"Fixture SDK","endpointPrefix":"fixture-prefix","arnNamespace":"fixture-arn"},"aws.auth#sigv4":{"name":"fixture-signing"},"aws.protocols#restJson1":{"noErrorWrapping":true},"aws.protocols#awsQueryCompatible":{},"smithy.api#xmlNamespace":{"uri":"urn:service","prefix":"svc"},"smithy.api#paginated":{"inputToken":"Token","outputToken":"Next","items":"Results.Entries","pageSize":"PageSize"}}},
"example#Fetch":{"type":"operation","input":{"target":"example#Input"},"output":{"target":"example#Output"},"errors":[{"target":"example#Problem"}],"traits":{"smithy.api#http":{"method":"POST","uri":"/things/{Id}","code":202},"smithy.api#endpoint":{"hostPrefix":"{Region}."},"smithy.api#httpChecksumRequired":true,"smithy.api#paginated":{"outputToken":"NextPage.Token","items":"Results.Items","pageSize":"PageSize"}}},
"example#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Token":{"target":"smithy.api#String"},"PageSize":{"target":"smithy.api#Integer","traits":{"smithy.api#range":{"max":37}}}}},
"example#Output":{"type":"structure","members":{"Next":{"target":"smithy.api#String"},"Results":{"target":"example#Result"},"NextPage":{"target":"example#Cursor"}}},
"example#Result":{"type":"structure","members":{"Entries":{"target":"smithy.api#String"},"Items":{"target":"smithy.api#String"}}},
"example#Cursor":{"type":"structure","members":{"Token":{"target":"smithy.api#String"}}},
"example#Problem":{"type":"structure","traits":{"smithy.api#error":"client"},"members":{"Message":{"target":"smithy.api#String"}}}
}}"#;
fn shape_index(
    service: inventory_core::catalog::Service<'_>,
    expected: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    service
        .shapes()?
        .enumerate()
        .find_map(|(index, shape)| match shape {
            Ok(shape) if shape.name().ok() == Some(expected) => Some(Ok(index)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or_else(|| format!("shape {expected} missing").into())
}

#[test]
fn service_protocol_and_identity_metadata_is_borrowed_from_compiled_catalog() -> TestResult {
    let (_source, bytes) = compile_model(MODEL, FIELDS)?;
    let service = fixture_service(&bytes)?;
    assert_eq!(service.version()?, "2025-04-03");
    assert_eq!(service.sdk_id()?, Some("Fixture SDK"));
    assert_eq!(service.endpoint_prefix()?, Some("fixture-prefix"));
    assert_eq!(service.arn_namespace()?, Some("fixture-arn"));
    assert_eq!(service.aws_query_compatible()?, true);
    assert_eq!(service.no_error_wrapping("aws.protocols#restJson1")?, true);
    assert_eq!(
        service.no_error_wrapping("aws.protocols#awsJson1_1")?,
        false
    );
    assert_eq!(
        service
            .wire()?
            .xml_namespace()?
            .map(|ns| (ns.uri, ns.prefix)),
        Some(("urn:service", Some("svc")))
    );
    assert_eq!(
        service.field_name(inventory_core::record::FieldId::new(2)?)?,
        "Id"
    );
    assert_eq!(
        service.field_name(inventory_core::record::FieldId::new(5)?)?,
        "Token"
    );
    assert_eq!(
        service.protocols()?.collect::<Result<Vec<_>, _>>()?,
        [
            "aws.protocols#awsQueryCompatible",
            "aws.protocols#restJson1"
        ]
    );
    Ok(())
}

#[test]
fn operation_metadata_merges_service_defaults_and_keeps_operation_overrides() -> TestResult {
    let (_source, bytes) = compile_model(MODEL, FIELDS)?;
    let service = fixture_service(&bytes)?;
    let mut operations = service.operations()?;
    let operation = operations.next().ok_or("operation missing")??;
    assert_eq!(operation.name()?, "Fetch");
    assert_eq!(
        operation.http()?,
        Some(HttpOperation {
            method: "POST",
            uri: "/things/{Id}",
            code: 202
        })
    );
    assert_eq!(operation.host_prefix()?, Some("{Region}."));
    assert_eq!(operation.checksum_required()?, true);
    let paginator = operation.paginator()?.ok_or("merged paginator missing")?;
    assert_eq!(paginator.input_token, Some("Token"));
    assert_eq!(paginator.output_token, Some("NextPage.Token"));
    assert_eq!(paginator.items, Some("Results.Items"));
    assert_eq!(paginator.page_size, Some("PageSize"));
    assert_eq!(paginator.max_page_size, Some(37));
    let errors = operation.errors()?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors.first().ok_or("error reference missing")?,
        &Target::Shape(shape_index(service, "example#Problem")?)
    );
    Ok(())
}

#[test]
fn absent_optional_service_and_operation_metadata_is_empty_or_none() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Read":{"type":"operation"}}}"#;
    let (_source, bytes) = compile_model(model, "")?;
    let service = fixture_service(&bytes)?;
    assert_eq!(service.sdk_id()?, None);
    assert_eq!(service.endpoint_prefix()?, None);
    assert_eq!(service.arn_namespace()?, None);
    assert_eq!(service.signing_name()?, None);
    assert_eq!(service.aws_query_compatible()?, false);
    assert_eq!(service.no_error_wrapping("aws.protocols#restJson1")?, false);
    assert_eq!(service.wire()?.xml_namespace()?, None);
    let mut operations = service.operations()?;
    let operation = operations.next().ok_or("operation missing")??;
    assert_eq!(operation.http()?, None);
    assert_eq!(operation.host_prefix()?, None);
    assert_eq!(operation.checksum_required()?, false);
    assert_eq!(operation.paginator()?, None);
    assert_eq!(operation.errors()?.count(), 0);
    Ok(())
}
