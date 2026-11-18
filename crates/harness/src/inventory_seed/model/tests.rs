use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn bytes() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog"),
    )
}

#[test]
fn given_real_sqs_map_output_when_path_compiled_then_queue_arn_is_a_named_map_key() -> TestResult {
    let bytes = bytes()?;
    let catalog = Catalog::borrow(&bytes)?;
    let operation = locate(catalog, "sqs", "GetQueueAttributes")?;
    let path = output::text_path(
        operation.service,
        operation.operation.output()?,
        &FieldPath::new(&["Attributes", "QueueArn"])?,
    )?;
    assert!(matches!(path.first(), Some(Selector::Field(_))));
    assert_eq!(path.get(1), Some(&Selector::Name(String::from("QueueArn"))));
    assert_eq!(path.len(), 2);
    assert!(
        output::text_path(
            operation.service,
            operation.operation.output()?,
            &FieldPath::new(&["Attributes"])?
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn given_real_ec2_list_output_when_path_compiled_then_index_is_checked_and_fields_are_modeled()
-> TestResult {
    let bytes = bytes()?;
    let catalog = Catalog::borrow(&bytes)?;
    let operation = locate(catalog, "ec2", "DescribeAvailabilityZones")?;
    let path = output::text_path(
        operation.service,
        operation.operation.output()?,
        &FieldPath::new(&["AvailabilityZones", "0", "ZoneName"])?,
    )?;
    assert_eq!(path.get(1), Some(&Selector::Index(0)));
    assert!(matches!(path.get(2), Some(Selector::Field(_))));
    assert!(
        output_path(
            operation.service,
            operation.operation.output()?,
            &FieldPath::new(&["AvailabilityZones", "256", "ZoneName"])?
        )
        .is_err()
    );
    assert!(
        output_path(
            operation.service,
            operation.operation.output()?,
            &FieldPath::new(&["AvailabilityZones", "0", "UnknownField"])?
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn given_real_unit_create_operation_when_request_identity_prepared_then_output_path_is_unnecessary()
-> TestResult {
    let bytes = bytes()?;
    let catalog = Catalog::borrow(&bytes)?;
    let operation = locate(catalog, "kms", "CreateAlias")?;
    let source = super::super::SeedIdentity::Request {
        input_field: FieldPath::new(&["AliasName"])?,
        template: Some(String::from(
            "arn:{partition}:kms:{region}:{account}:{value}",
        )),
    };
    let identity = identity::prepare(operation, &source)?;
    assert!(
        matches!(identity, PreparedIdentity::Request { input_field, template: Some(_) } if input_field.len() == 1)
    );
    let invalid = super::super::SeedIdentity::Request {
        input_field: FieldPath::new(&["Missing"])?,
        template: None,
    };
    assert!(identity::prepare(operation, &invalid).is_err());
    Ok(())
}

#[test]
fn given_nontext_modeled_request_identity_when_prepared_then_mutation_is_rejected_before_execution()
-> TestResult {
    let bytes = bytes()?;
    let catalog = Catalog::borrow(&bytes)?;
    let operation = locate(catalog, "sqs", "CreateQueue")?;
    let source = super::super::SeedIdentity::Request {
        input_field: FieldPath::new(&["Attributes"])?,
        template: None,
    };
    assert!(identity::prepare(operation, &source).is_err());
    Ok(())
}
