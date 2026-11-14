use super::*;

#[test]
fn irrelevant_fields_are_ignored_for_unknown_and_known_kinds()
-> Result<(), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(
        r#"{"shapes":{
      "n#Svc":{"type":"service","input":42,"members":false},
      "n#Future":{"type":"future","input":42,"members":[1,2],"traits":{"aws.api#arn":{}}},
      "n#String":{"type":"string","input":42,"members":false,"traits":{"aws.api#arn":{}}},
      "n#R":{"type":"resource","input":42,"members":"ignored"},
      "n#S":{"type":"structure","input":42,"members":{"A":{}}},
      "n#U":{"type":"union","input":42,"members":{"B":{}}},
      "n#ListIgnored":{"type":"operation","members":42}
    }}"#,
    )?;

    assert_eq!(
        compute_stats(&model),
        ModelStats {
            operations: 1,
            read_named: 1,
            list_describe: 1,
            list_describe_root: 1,
            resources: 1,
            arn_templates: 2,
            members: 2,
            ..ModelStats::default()
        }
    );
    let mut totals = Totals::default();
    totals.add_model(model, Path::new("irrelevant.json"))?;
    assert_eq!(
        totals.distinct_member_names,
        ["A".into(), "B".into()].into()
    );
    Ok(())
}

#[test]
fn required_members_exclude_roots_regardless_of_member_position()
-> Result<(), Box<dyn std::error::Error>> {
    let model: ModelFile = serde_json::from_str(
        r#"{"shapes":{
      "n#Svc":{"type":"service"},
      "n#ListFirst":{"type":"operation","input":{"target":"n#First"}},
      "n#DescribeLast":{"type":"operation","input":{"target":"n#Last"}},
      "n#First":{"type":"structure","members":{"A":{"traits":{"smithy.api#required":{}}},"Z":{}}},
      "n#Last":{"type":"structure","members":{"A":{},"Z":{"traits":{"smithy.api#required":{}}}}}
    }}"#,
    )?;

    assert_eq!(
        compute_stats(&model),
        ModelStats {
            operations: 2,
            read_named: 2,
            list_describe: 2,
            members: 4,
            ..ModelStats::default()
        }
    );
    Ok(())
}

#[test]
fn identical_reports_have_no_differences() -> Result<(), Box<dyn std::error::Error>> {
    let report = "services 1 \r\noperations 2\n";

    assert_eq!(differences(report, report)?, "");
    assert_eq!(differences("", "")?, "");
    Ok(())
}
