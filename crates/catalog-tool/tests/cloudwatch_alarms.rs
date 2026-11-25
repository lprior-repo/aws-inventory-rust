use catalog_tool::{
    overlay::{self, MemberPath},
    smithy::Registry,
};
use inventory_core::catalog::runtime::{
    BindingSource, BindingValue, IdentityPlan, ItemPlan, PlanPurpose, PlanScope, RuntimeCatalog,
};
use inventory_core::record::FieldId;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CATALOG: &[u8] = include_bytes!("../../../catalog/aws-api.catalog");
const FIELDS: &str = include_str!("../../../catalog/field-ids.txt");

#[test]
fn given_the_pinned_catalog_when_describing_alarms_then_each_list_emits_a_distinct_arn_keyed_kind()
-> TestResult {
    let runtime = RuntimeCatalog::from_bytes(CATALOG)?;
    let fields = Registry::parse(FIELDS)?;
    let plan = runtime
        .lookup("cloudwatch", "DescribeAlarms")
        .ok_or("DescribeAlarms is not callable")?;
    assert_eq!(plan.purpose(), &PlanPurpose::Enumeration);
    assert_eq!(plan.scope(), PlanScope::Region);
    let arn = IdentityPlan::Arn(Box::new([field(&fields, "AlarmArn")?]));
    let expected = [
        ("primary", "AWS::CloudWatch::Alarm", "MetricAlarms"),
        (
            "composite_alarms",
            "AWS::CloudWatch::CompositeAlarm",
            "CompositeAlarms",
        ),
        ("log_alarms", "AWS::CloudWatch::LogAlarm", "LogAlarms"),
    ];
    let emissions = plan.resource_emissions();
    assert_eq!(emissions.len(), expected.len());
    for (key, kind, list) in expected {
        let emission = emissions
            .iter()
            .find(|emission| emission.key() == key)
            .ok_or_else(|| format!("missing alarm emission {key}"))?;
        assert_eq!(emission.kind().as_str(), kind);
        assert_eq!(
            emission.items(),
            &ItemPlan::Path(Box::new([field(&fields, list)?]))
        );
        assert_eq!(emission.identity(), &arn);
    }
    Ok(())
}

#[test]
fn given_the_alarm_enumerator_when_loaded_then_it_requests_all_types_and_omits_evaluated_content()
-> TestResult {
    let runtime = RuntimeCatalog::from_bytes(CATALOG)?;
    let fields = Registry::parse(FIELDS)?;
    let plan = runtime
        .lookup("cloudwatch", "DescribeAlarms")
        .ok_or("DescribeAlarms is not callable")?;
    let alarm_types = field(&fields, "AlarmTypes")?;
    let types = plan
        .bindings()
        .iter()
        .find(|binding| binding.path.as_ref() == [alarm_types])
        .ok_or("AlarmTypes binding is missing")?;
    let BindingSource::Fixed(BindingValue::Strings(values)) = &types.source else {
        return Err("AlarmTypes is not a fixed string list".into());
    };
    assert_eq!(
        values.as_ref(),
        ["MetricAlarm", "CompositeAlarm", "LogAlarm"]
    );
    let next = field(&fields, "NextToken")?;
    let [token] = plan.pagination().tokens.as_ref() else {
        return Err("expected one continuation token".into());
    };
    assert_eq!(token.request_path.as_ref(), [next]);
    assert_eq!(token.response_path.as_ref(), [next]);
    let overlays = overlay::embedded()?;
    let describe = overlays
        .iter()
        .find(|service| service.service() == "cloudwatch")
        .and_then(|service| service.operations().get("DescribeAlarms"))
        .ok_or("DescribeAlarms overlay is missing")?;
    let listed = describe
        .exclude_response_members
        .iter()
        .map(MemberPath::as_str)
        .collect::<Vec<_>>();
    for path in [
        "MetricAlarms.StateReason",
        "MetricAlarms.StateReasonData",
        "CompositeAlarms.StateReason",
        "CompositeAlarms.StateReasonData",
        "LogAlarms.StateReason",
        "LogAlarms.StateReasonData",
        "LogAlarms.ScheduledQueryConfiguration",
    ] {
        assert!(listed.contains(&path), "does not exclude {path}");
    }
    let excluded = plan.excluded_members();
    for name in [
        "StateReason",
        "StateReasonData",
        "ScheduledQueryConfiguration",
    ] {
        assert!(
            excluded.contains(field(&fields, name)?),
            "does not exclude {name}"
        );
    }
    assert!(!excluded.contains(field(&fields, "AlarmArn")?));
    Ok(())
}

fn field(registry: &Registry, name: &str) -> TestResult<FieldId> {
    registry
        .id(name)
        .ok_or_else(|| format!("missing field {name}").into())
}
