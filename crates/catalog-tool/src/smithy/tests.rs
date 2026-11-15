use std::path::Path;

use super::{
    model::ModelFile,
    report::render_report,
    stats::{ModelStats, compute_stats},
    totals::Totals,
};

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
 "n#Svc":{"type":"service","traits":{"aws.protocols#restJson1":{},"smithy.protocols#rpcv2Cbor":{}}},
 "n#ListThings":{"type":"operation","traits":{"smithy.api#readonly":{},"smithy.api#paginated":{}}},
 "n#DescribeThing":{"type":"operation","input":{"target":"n#Input"}},
 "n#GetThing":{"type":"operation","traits":{"smithy.api#readonly":{}}},"n#DeleteThing":{"type":"operation"},
 "n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
 "n#Choice":{"type":"union","members":{"Id":{"target":"smithy.api#String"},"Name":{"target":"smithy.api#String"}}},
 "n#Resource":{"type":"resource"},"n#Arn":{"type":"string","traits":{"aws.api#arn":"arn:test"}},
 "n#Unknown":{"type":"futureType","futureField":{"ignored":true}}
}}"#;

#[test]
fn every_per_model_counter_matches_the_contract() -> Result<(), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(MODEL)?;

    let stats = compute_stats(&model)?;

    assert_eq!(
        stats,
        ModelStats {
            operations: 4,
            read_named: 3,
            readonly: 2,
            list_describe: 2,
            list_describe_root: 1,
            paginated: 1,
            resources: 1,
            arn_templates: 1,
            members: 3,
        }
    );
    let mut totals = Totals::default();
    totals.add_model(model, Path::new("test.json"))?;
    assert_eq!(
        totals.protocol_counts,
        [
            ("aws.protocols#restJson1".into(), 1),
            ("smithy.protocols#rpcv2Cbor".into(), 1)
        ]
        .into()
    );
    assert_eq!(
        totals.distinct_member_names,
        ["Id".into(), "Name".into()].into()
    );
    Ok(())
}

#[test]
fn malformed_operation_inputs_fail_before_stats() -> Result<(), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(
        r#"{"shapes":{
      "n#Svc":{"type":"service"},"n#ListAbsent":{"type":"operation"},
      "n#ListUnit":{"type":"operation","input":{"target":"smithy.api#Unit"}},
      "n#ListMissing":{"type":"operation","input":{"target":"missing#Required"}},
      "n#ListSuffix":{"type":"operation","input":{"target":"Required"}},
      "n#ListOptional":{"type":"operation","input":{"target":"n#Optional"}},
      "n#DescribeUnion":{"type":"operation","input":{"target":"n#Union"}},
      "n#DescribeRequired":{"type":"operation","input":{"target":"other#Required"}},
      "other#Required":{"type":"structure","members":{"Id":{"traits":{"smithy.api#required":{}}}}},
      "smithy.api#Unit":{"type":"structure","members":{"Id":{"traits":{"smithy.api#required":{}}}}},
      "n#Optional":{"type":"structure","members":{"Id":{}}},
      "n#Union":{"type":"union","members":{"Id":{"traits":{"smithy.api#required":{}}}}}
    }}"#,
    )?;

    let result = super::stats::validate_inputs(&model, Path::new("service.json"));
    assert!(matches!(
        result,
        Err(super::Error::InvalidInput { source, .. })
            if source.target == "n#Union"
                && source.kind == crate::model_source::InputFailure::NonStructure
    ));
    Ok(())
}

#[test]
fn read_prefixes_are_case_sensitive() -> Result<(), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(
        r#"{"shapes":{
      "n#Svc":{"type":"service"},"n#ListA":{"type":"operation"},"n#DescribeA":{"type":"operation"},
      "n#GetA":{"type":"operation"},"n#BatchGetA":{"type":"operation"},"n#SearchA":{"type":"operation"},
      "n#ScanA":{"type":"operation"},"n#QueryA":{"type":"operation"},"n#LookupA":{"type":"operation"},
      "n#SelectA":{"type":"operation"},"n#listA":{"type":"operation"},"n#LIST_A":{"type":"operation"},
      "n#describeA":{"type":"operation"},"n#getA":{"type":"operation"},"n#BatchgetA":{"type":"operation"}
    }}"#,
    )?;

    let stats = compute_stats(&model)?;

    assert_eq!(
        (
            stats.operations,
            stats.read_named,
            stats.list_describe,
            stats.list_describe_root
        ),
        (14, 9, 2, 2)
    );
    Ok(())
}

#[test]
fn unknown_shapes_and_fields_do_not_add_counters() -> Result<(), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(
        r#"{"unknown":42,"shapes":{
      "n#Svc":{"type":"service","future":true},"n#X":{"type":"future","future":{"x":1}}
    }}"#,
    )?;

    assert_eq!(compute_stats(&model)?, ModelStats::default());
    Ok(())
}

#[test]
fn rendering_is_the_complete_fixed_literal() -> Result<(), Box<dyn std::error::Error>> {
    let totals = Totals {
        services: 1,
        services_with_root: 7,
        counts: ModelStats {
            operations: 2,
            read_named: 3,
            readonly: 4,
            list_describe: 5,
            list_describe_root: 6,
            paginated: 8,
            resources: 9,
            arn_templates: 10,
            members: 11,
        },
        distinct_member_names: (0..12).map(|index| index.to_string()).collect(),
        json_services: 13,
        xml_only_services: 14,
        cbor_services: 15,
        protocol_counts: [
            ("smithy.protocols#rpcv2Cbor".into(), 19),
            ("aws.protocols#restJson1".into(), 17),
        ]
        .into(),
    };

    assert_eq!(
        render_report(&totals)?,
        concat!(
            "services 1\noperations 2\nread_named 3\nreadonly 4\nlist_describe 5\n",
            "list_describe_root 6\nservices_with_root 7\npaginated 8\nresources 9\narn_templates 10\n",
            "members 11\ndistinct_member_names 12\njson_services 13\nxml_only_services 14\ncbor_services 15\n",
            "protocol aws.protocols#restJson1 17\nprotocol smithy.protocols#rpcv2Cbor 19\n"
        )
    );
    Ok(())
}

mod totals;

mod fields;
