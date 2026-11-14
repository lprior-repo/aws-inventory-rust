use super::*;

const JSON: &str = r#"{"shapes":{
 "n#Svc":{"type":"service","traits":{"aws.protocols#restJson1":{},"aws.protocols#awsJson1_0":{}}},
 "n#List":{"type":"operation","traits":{"smithy.api#readonly":{},"smithy.api#paginated":{}}},
 "n#S":{"type":"structure","members":{"Shared":{},"Unique1":{}}},
 "n#R":{"type":"resource","traits":{"aws.api#arn":{}}}
}}"#;
const CBOR_QUERY: &str = r#"{"shapes":{
 "n#Svc":{"type":"service","traits":{"smithy.protocols#rpcv2Cbor":{},"aws.protocols#awsQuery":{}}},
 "n#Describe":{"type":"operation","input":{"target":"n#S"}},
 "n#S":{"type":"structure","members":{"Shared":{"traits":{"smithy.api#required":{}}},"Unique2":{}}},
 "n#U":{"type":"union","members":{"Shared":{}}}
}}"#;

#[test]
fn two_models_sum_counters_and_share_one_distinct_name_set()
-> Result<(), Box<dyn std::error::Error>> {
    let mut totals = Totals::default();
    totals.add_model(serde_json::from_str(JSON)?, Path::new("a.json"))?;

    totals.add_model(serde_json::from_str(CBOR_QUERY)?, Path::new("b.json"))?;

    assert_eq!(
        totals.counts,
        ModelStats {
            operations: 2,
            read_named: 2,
            readonly: 1,
            list_describe: 2,
            list_describe_root: 1,
            paginated: 1,
            resources: 1,
            arn_templates: 1,
            members: 5,
        }
    );
    assert_eq!((totals.services, totals.services_with_root), (2, 1));
    assert_eq!(
        totals.distinct_member_names,
        ["Shared".into(), "Unique1".into(), "Unique2".into()].into()
    );
    assert_eq!(
        (
            totals.json_services,
            totals.xml_only_services,
            totals.cbor_services
        ),
        (1, 0, 1)
    );
    assert_eq!(
        totals.protocol_counts,
        [
            ("aws.protocols#awsJson1_0".into(), 1),
            ("aws.protocols#awsQuery".into(), 1),
            ("aws.protocols#restJson1".into(), 1),
            ("smithy.protocols#rpcv2Cbor".into(), 1),
        ]
        .into()
    );
    Ok(())
}

#[test]
fn protocol_classes_count_services_not_traits() -> Result<(), Box<dyn std::error::Error>> {
    let mut totals = Totals::default();
    totals.add_model(serde_json::from_str(JSON)?, Path::new("a.json"))?;
    totals.add_model(serde_json::from_str(CBOR_QUERY)?, Path::new("b.json"))?;

    totals.add_model(serde_json::from_str(r#"{"shapes":{"n#Svc":{"type":"service","traits":{"aws.protocols#awsQuery":{},"aws.protocols#awsQueryCompatible":{}}}}}"#)?, Path::new("c.json"))?;
    totals.add_model(serde_json::from_str(JSON)?, Path::new("d.json"))?;

    assert_eq!(
        (
            totals.services,
            totals.json_services,
            totals.xml_only_services,
            totals.cbor_services
        ),
        (4, 2, 1, 1)
    );
    assert_eq!(
        totals.protocol_counts,
        [
            ("aws.protocols#awsJson1_0".into(), 2),
            ("aws.protocols#awsQuery".into(), 2),
            ("aws.protocols#awsQueryCompatible".into(), 1),
            ("aws.protocols#restJson1".into(), 2),
            ("smithy.protocols#rpcv2Cbor".into(), 1),
        ]
        .into()
    );
    Ok(())
}

type ClassificationCase = (&'static str, (usize, usize, usize), &'static [&'static str]);

const CLASSIFICATION_CASES: &[ClassificationCase] = &[
    (
        r#"{"aws.protocols#awsJson1_1":{}}"#,
        (1, 0, 0),
        &["aws.protocols#awsJson1_1"],
    ),
    (
        r#"{"aws.protocols#ec2Query":{}}"#,
        (0, 1, 0),
        &["aws.protocols#ec2Query"],
    ),
    (
        r#"{"aws.protocols#restXml":{}}"#,
        (0, 1, 0),
        &["aws.protocols#restXml"],
    ),
    (
        r#"{"aws.protocols#restJson1":{},"aws.protocols#awsQuery":{}}"#,
        (1, 0, 0),
        &["aws.protocols#restJson1", "aws.protocols#awsQuery"],
    ),
    (
        r#"{"custom.protocols#future":{},"example#metadata":{}}"#,
        (0, 0, 0),
        &["custom.protocols#future"],
    ),
];

#[test]
fn every_protocol_class_and_custom_protocol_trait_is_exact()
-> Result<(), Box<dyn std::error::Error>> {
    CLASSIFICATION_CASES
        .iter()
        .try_for_each(|(traits, expected, ids)| {
            let traits: serde_json::Value = serde_json::from_str(traits)?;
            let document =
                serde_json::json!({"shapes":{"n#Svc":{"type":"service","traits":traits}}});
            let mut totals = Totals::default();

            totals.add_model(
                serde_json::from_value(document)?,
                Path::new("protocol.json"),
            )?;
            assert_eq!(
                (
                    totals.json_services,
                    totals.xml_only_services,
                    totals.cbor_services
                ),
                *expected
            );
            assert_eq!(
                totals.protocol_counts,
                ids.iter().map(|id| ((*id).to_owned(), 1)).collect()
            );
            assert_eq!(totals.services, 1);
            Ok::<_, Box<dyn std::error::Error>>(())
        })
}
